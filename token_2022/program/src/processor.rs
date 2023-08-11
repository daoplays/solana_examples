use crate::accounts;
use crate::state;
use crate::utils;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::rent,
};

use spl_token::{instruction, state::Mint};

use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use crate::error::NewError;
use crate::instruction::{CreateMeta, TokenInstruction};

pub struct Processor;
impl Processor {
    pub fn process<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = TokenInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            TokenInstruction::CreateToken { metadata } => {
                Self::create_token(program_id, accounts, metadata)
            }
        }
    }

    pub fn create_program_account<'a>(
        funding_account: &AccountInfo<'a>,
        pda: &AccountInfo<'a>,
        program_id: &Pubkey,
        bump_seed: u8,
        data_size: usize,
        seed: &[u8],
    ) -> ProgramResult {
        // Check if the account has already been initialized
        if **pda.try_borrow_lamports()? > 0 {
            msg!("This account is already initialized. skipping");
            return Ok(());
        }

        msg!("Creating program derived account");

        let space: u64 = data_size.try_into().unwrap();
        let lamports = rent::Rent::default().minimum_balance(data_size);

        msg!("Require {} lamports for {} size data", lamports, data_size);
        let ix = solana_program::system_instruction::create_account(
            funding_account.key,
            pda.key,
            lamports,
            space,
            program_id,
        );

        // Sign and submit transaction
        invoke_signed(
            &ix,
            &[funding_account.clone(), pda.clone()],
            &[&[seed, &[bump_seed]]],
        )?;

        Ok(())
    }

    fn create_token<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        metadata: CreateMeta,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed eight accounts, get them all first and then check their value is as expected
        let funding_account_info = next_account_info(account_info_iter)?;
        let token_mint_account_info = next_account_info(account_info_iter)?;
        let new_token_account: &AccountInfo<'_> = next_account_info(account_info_iter)?;

        let transfer_hook_program_account: &AccountInfo<'_> = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        if !funding_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        accounts::check_token_account(
            funding_account_info,
            token_mint_account_info,
            new_token_account,
        )?;

        accounts::check_token_program_2022_key(token_program_account_info)?;
        accounts::check_associated_token_program_key(associated_token_account_info)?;
        accounts::check_system_program_key(system_program_account_info)?;

        if metadata.extensions == 0 {
            return Ok(());
        }

        // support the following extensions
        // transfer fee - 0
        // permanent delegate - 1
        // interest bearing - 2
        // non transferable - 4
        // default account state - 8

        let transfer = state::Extensions::TransferFee as u8;
        let delegate = state::Extensions::PermanentDelegate as u8;
        let interest = state::Extensions::InterestBearing as u8;
        let transferable = state::Extensions::NonTransferable as u8;
        let default = state::Extensions::DefaultState as u8;
        let transfer_hook = state::Extensions::TransferHook as u8;

        let include_transfer: bool = metadata.extensions & transfer > 0;
        let include_delegate: bool = metadata.extensions & delegate > 0;
        let include_interest: bool = metadata.extensions & interest > 0;
        let include_transferable: bool = metadata.extensions & transferable > 0;
        let include_default_state: bool = metadata.extensions & default > 0;
        let include_transfer_hook: bool = metadata.extensions & transfer_hook > 0;

        msg!(
            "include : {} {} {} {} {} {}",
            include_transfer,
            include_delegate,
            include_interest,
            include_transferable,
            include_default_state,
            include_transfer_hook
        );

        let mut extension_types: Vec<spl_token_2022::extension::ExtensionType> = Vec::new();
        if include_transfer {
            extension_types.push(spl_token_2022::extension::ExtensionType::TransferFeeConfig);
        }
        if include_delegate {
            extension_types.push(spl_token_2022::extension::ExtensionType::PermanentDelegate);
        }
        if include_interest {
            extension_types.push(spl_token_2022::extension::ExtensionType::InterestBearingConfig);
        }
        if include_transferable {
            extension_types.push(spl_token_2022::extension::ExtensionType::NonTransferable);
        }
        if include_default_state {
            extension_types.push(spl_token_2022::extension::ExtensionType::DefaultAccountState);
        }
        if include_transfer_hook {
            extension_types.push(spl_token_2022::extension::ExtensionType::TransferHook);
        }

        let space = spl_token_2022::extension::ExtensionType::get_account_len::<
            spl_token_2022::state::Mint,
        >(&extension_types);
        // first create the mint account for the new NFT
        let mint_rent = rent::Rent::default().minimum_balance(space);

        msg!("create account");
        let create_idx = solana_program::system_instruction::create_account(
            &funding_account_info.key,
            &token_mint_account_info.key,
            mint_rent,
            space as u64,
            &spl_token_2022::id(),
        );

        invoke(
            &create_idx,
            &[
                funding_account_info.clone(),
                token_mint_account_info.clone(),
            ],
        )?;

        if include_transfer {
            msg!(
                "init transfer config {} {}",
                metadata.transfer_fee_bp,
                metadata.transfer_fee_max
            );
            let config_init_idx =
            spl_token_2022::extension::transfer_fee::instruction::initialize_transfer_fee_config(
                &spl_token_2022::ID,
                &token_mint_account_info.key,
                Some(&funding_account_info.key),
                Some(&funding_account_info.key),
                metadata.transfer_fee_bp,
                metadata.transfer_fee_max,
            )
            .unwrap();

            invoke(
                &config_init_idx,
                &[
                    token_program_account_info.clone(),
                    token_mint_account_info.clone(),
                    funding_account_info.clone(),
                ],
            )?;
        }

        if include_delegate {
            msg!("init delegate config");
            let config_init_idx = spl_token_2022::instruction::initialize_permanent_delegate(
                &token_program_account_info.key,
                &token_mint_account_info.key,
                &funding_account_info.key,
            )
            .unwrap();

            invoke(
                &config_init_idx,
                &[
                    token_program_account_info.clone(),
                    token_mint_account_info.clone(),
                    funding_account_info.clone(),
                ],
            )?;
        }

        if include_interest {
            msg!("init interest config {}", metadata.interest_rate);
            let config_init_idx =
                spl_token_2022::extension::interest_bearing_mint::instruction::initialize(
                    &spl_token_2022::ID,
                    &token_mint_account_info.key,
                    Some(*funding_account_info.key),
                    metadata.interest_rate,
                )
                .unwrap();

            invoke(
                &config_init_idx,
                &[
                    token_program_account_info.clone(),
                    token_mint_account_info.clone(),
                    funding_account_info.clone(),
                ],
            )?;
        }

        if include_transferable {
            msg!("init non-transferable config");
            let config_init_idx = spl_token_2022::instruction::initialize_non_transferable_mint(
                &spl_token_2022::id(),
                &token_mint_account_info.key,
            )
            .unwrap();

            invoke(
                &config_init_idx,
                &[
                    token_program_account_info.clone(),
                    token_mint_account_info.clone(),
                ],
            )?;
        }

        if include_default_state {
            msg!("init default config");
            let config_init_idx =
            spl_token_2022::extension::default_account_state::instruction::initialize_default_account_state(
                &spl_token_2022::ID,
                &token_mint_account_info.key,
                &spl_token_2022::state::AccountState::Frozen
            )
            .unwrap();

            invoke(
                &config_init_idx,
                &[
                    token_program_account_info.clone(),
                    token_mint_account_info.clone(),
                    funding_account_info.clone(),
                ],
            )?;
        }

        if include_transfer_hook {
            msg!("init transfer hook");
            let config_init_idx =
                spl_token_2022::extension::transfer_hook::instruction::initialize(
                    &spl_token_2022::ID,
                    &token_mint_account_info.key,
                    Some(*funding_account_info.key),
                    Some(*transfer_hook_program_account.key),
                )
                .unwrap();

            invoke(
                &config_init_idx,
                &[
                    token_program_account_info.clone(),
                    token_mint_account_info.clone(),
                    funding_account_info.clone(),
                    transfer_hook_program_account.clone(),
                ],
            )?;
        }

        msg!("initialise mint");
        let mint_idx = spl_token_2022::instruction::initialize_mint2(
            &spl_token_2022::id(),
            &token_mint_account_info.key,
            &funding_account_info.key,
            Some(&funding_account_info.key),
            3,
        )
        .unwrap();

        invoke(
            &mint_idx,
            &[
                token_program_account_info.clone(),
                token_mint_account_info.clone(),
                funding_account_info.clone(),
            ],
        )?;

        msg!("create ata");

        let create_ata_idx = create_associated_token_account(
            &funding_account_info.key,
            &funding_account_info.key,
            &token_mint_account_info.key,
            &spl_token_2022::ID,
        );

        invoke(
            &create_ata_idx,
            &[
                funding_account_info.clone(),
                new_token_account.clone(),
                funding_account_info.clone(),
                token_mint_account_info.clone(),
                token_program_account_info.clone(),
            ],
        )?;

        if !include_default_state {
            msg!("mint");

            let mint_to_idx = spl_token_2022::instruction::mint_to_checked(
                &spl_token_2022::ID,
                &token_mint_account_info.key,
                &new_token_account.key,
                &funding_account_info.key,
                &[&funding_account_info.key],
                1000000,
                3,
            )
            .unwrap();

            invoke(
                &mint_to_idx,
                &[
                    token_program_account_info.clone(),
                    token_mint_account_info.clone(),
                    new_token_account.clone(),
                    funding_account_info.clone(),
                ],
            )?;
        }
        Ok(())
    }
}
