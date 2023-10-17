use crate::accounts;
use crate::instruction::TransferHookInstruction;
use crate::state;
use crate::utils;
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::rent,
    instruction::AccountMeta

};

use spl_associated_token_account::instruction::create_associated_token_account;

use crate::instruction::{CreateMeta, TransferMeta, TokenInstruction};

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
            },
            TokenInstruction::Transfer { metadata} => {
                Self::transfer(program_id, accounts, metadata)
            }
        }
    }

    fn create_token<'a>(
        _program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        metadata: CreateMeta,
    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed eight accounts, get them all first and then check their value is as expected
        let funding_account_info = next_account_info(account_info_iter)?;
        let token_mint_account_info = next_account_info(account_info_iter)?;
        let new_token_account: &AccountInfo<'_> = next_account_info(account_info_iter)?;

        let transfer_hook_program_account: &AccountInfo<'_> = next_account_info(account_info_iter)?;
        let transfer_hook_validation_account: &AccountInfo<'_> = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        // we may additionally pass an account to hold data for the mint
        let mint_data_option: Option<&AccountInfo<'_>> = account_info_iter.next();


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
        // transfer fee - 1
        // permanent delegate - 2
        // interest bearing - 4
        // non transferable - 8
        // default account state - 16
        // transfer hook - 32

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

        let space = spl_token_2022::extension::ExtensionType::try_calculate_account_len::<
            spl_token_2022::state::Mint,
        >(&extension_types).unwrap();
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

           
            
            let mut account_metas = vec![
                solana_program::instruction::AccountMeta::new(*transfer_hook_validation_account.key, false),
                solana_program::instruction::AccountMeta::new(*token_mint_account_info.key, false),
                solana_program::instruction::AccountMeta::new(*funding_account_info.key, true),
                solana_program::instruction::AccountMeta::new_readonly(*system_program_account_info.key, false),
            ];

            let mut account_infos = vec![ 
                transfer_hook_validation_account.clone(),
                token_mint_account_info.clone(),
                funding_account_info.clone(),
                system_program_account_info.clone()
            ];

            // check if we added a mint data account
            if mint_data_option.is_some() {
                let mint_data_account_info = mint_data_option.unwrap();
                let mint_data_meta = solana_program::instruction::AccountMeta::new(*mint_data_account_info.key, false);
                account_metas.push(mint_data_meta);

                account_infos.push(mint_data_account_info.clone());
            }

            let instruction_data = TransferHookInstruction::InitializeExtraAccountMetas.pack();

            let init_accounts_idx = solana_program::instruction::Instruction {
                program_id: *transfer_hook_program_account.key,
                accounts: account_metas,
                data: instruction_data,
            };
            
            invoke(
                &init_accounts_idx,
                &account_infos,
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

    fn transfer<'a>(
        _program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        metadata: TransferMeta,
    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();

        let funding_account_info = next_account_info(account_info_iter)?;
        let source_token_account_info = next_account_info(account_info_iter)?;
        let dest_token_account_info = next_account_info(account_info_iter)?;
        let mint_account_info = next_account_info(account_info_iter)?;

        let hook_program_account_info = next_account_info(account_info_iter)?;
        let validation_account_info = next_account_info(account_info_iter)?;
        let mint_data_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;

        // verify that the accounts are what we expect
        accounts::check_token_account(
            funding_account_info,
            mint_account_info,
            source_token_account_info,
        )?;

        let _validation_bump_seed = utils::check_program_data_account(validation_account_info, hook_program_account_info.key,vec![utils::EXTRA_ACCOUNT_METAS_SEED, &mint_account_info.key.to_bytes()], "validation".to_string()).unwrap();

        let _mint_data_bump_seed = utils::check_program_data_account(mint_data_account_info, hook_program_account_info.key,vec![b"mint_data", &mint_account_info.key.to_bytes()], "mint_data".to_string()).unwrap();

        accounts::check_token_program_2022_key(token_program_account_info)?;

        // create a transfer_checked instruction
        // this needs to be mutable because we will manually add extra accounts
        let mut transfer_idx = spl_token_2022::instruction::transfer_checked(
            &spl_token_2022::id(),
            &source_token_account_info.key,
            &mint_account_info.key,
            &dest_token_account_info.key,
            &funding_account_info.key,
            &[&funding_account_info.key],
            metadata.amount,
            3,
        ).unwrap();
    
        // add the three transfer hook accounts
        transfer_idx.accounts.push(AccountMeta::new_readonly(
            *hook_program_account_info.key,
            false,
        ));
    
        transfer_idx.accounts.push(AccountMeta::new_readonly(
            *validation_account_info.key,
            false,
        ));
    
        transfer_idx.accounts.push(AccountMeta::new(
            *mint_data_account_info.key,
            false,
        ));

        invoke(
            &transfer_idx,
            &[
                token_program_account_info.clone(),
                source_token_account_info.clone(),
                mint_account_info.clone(),
                dest_token_account_info.clone(),
                funding_account_info.clone(),
                hook_program_account_info.clone(),
                validation_account_info.clone(),
                mint_data_account_info.clone(),
            ],
        )?;

        Ok(())
    }
}
