use crate::{
    accounts,
    error::OptionsError,
    instruction::{
        accounts::{
            CreateCollectionAccounts, CreateOptionAccounts, ExecuteAccounts, InitAccounts,
            ListAccounts, PurchaseAccounts, RefundAccounts,
        },
        ListOptionAssetMeta,
    },
    utils,
};
use borsh::{to_vec, BorshDeserialize};
use solana_program::{
    account_info::next_account_info,
    clock::Clock,
    native_token::LAMPORTS_PER_SOL,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    system_instruction,
    sysvar::{rent, Sysvar},
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};
use solana_program::{log::sol_log_compute_units, serde_varint::serialize};

use crate::instruction::{CreateOptionAssetMeta, CreateOptionCollectionMeta, OptionsInstruction};
use mpl_core::instructions::{CreateCollectionV1CpiBuilder, CreateV1CpiBuilder};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};

pub struct Processor;
impl Processor {
    pub fn process<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Process");
        let instruction = OptionsInstruction::try_from_slice(instruction_data)?;

        match instruction {
            OptionsInstruction::Init() => {
                msg!("Init");
                Self::init(program_id, accounts)
            }

            OptionsInstruction::CreateCollection(args) => {
                msg!("Create Collection");
                Self::create_collection(program_id, accounts, args)
            }

            OptionsInstruction::CreateOption(args) => {
                Self::create_option(program_id, accounts, args)
            }
            OptionsInstruction::TransferToken => utils::transfer(program_id, accounts),

            OptionsInstruction::Execute() => Self::execute(program_id, accounts),

            OptionsInstruction::Refund() => Self::refund(program_id, accounts),

            OptionsInstruction::List(args) => Self::list(program_id, accounts, args),
            OptionsInstruction::Purchase() => Self::purchase(program_id, accounts),
        }
    }

    pub fn init<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
        let ctx = InitAccounts::context(accounts)?;

        if !ctx.accounts.user.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (_pda_account, bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

        let lamports = rent::Rent::default().minimum_balance(0);

        let ix = solana_program::system_instruction::create_account(
            ctx.accounts.user.key,
            ctx.accounts.program_pda.key,
            lamports,
            0,
            program_id,
        );

        invoke_signed(
            &ix,
            &[ctx.accounts.user.clone(), ctx.accounts.program_pda.clone()],
            &[&[b"pda", &[bump_seed]]],
        )?;

        Ok(())
    }

    fn create_collection<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        args: CreateOptionCollectionMeta,
    ) -> ProgramResult {
        let ctx = CreateCollectionAccounts::context(accounts)?;

        let (_collection_account, bump_seed) = Pubkey::find_program_address(
            &[&ctx.accounts.token_mint.key.to_bytes(), b"Collection"],
            &program_id,
        );

        let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

        let _create_cpi = CreateCollectionV1CpiBuilder::new(ctx.accounts.core_program)
            .collection(ctx.accounts.collection_account)
            .payer(ctx.accounts.user)
            .update_authority(Some(ctx.accounts.program_pda))
            .name(args.name.to_string())
            .uri(args.uri.to_string())
            .system_program(ctx.accounts.system_program)
            .invoke_signed(&[
                &[
                    &ctx.accounts.token_mint.key.to_bytes(),
                    b"Collection",
                    &[bump_seed],
                ],
                &[b"pda", &[pda_bump_seed]],
            ])
            .unwrap();

        mpl_core::instructions::AddCollectionPluginV1CpiBuilder::new(ctx.accounts.core_program)
            .collection(ctx.accounts.collection_account)
            .payer(ctx.accounts.user)
            .authority(Some(ctx.accounts.program_pda))
            .plugin(mpl_core::types::Plugin::Attributes(
                mpl_core::types::Attributes {
                    attribute_list: vec![mpl_core::types::Attribute {
                        key: "token_mint".to_string(),
                        value: ctx.accounts.token_mint.key.to_string(),
                    }],
                },
            ))
            .system_program(ctx.accounts.system_program)
            .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
            .unwrap();

        Ok(())
    }

    fn create_option<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        args: CreateOptionAssetMeta,
    ) -> ProgramResult {
        let ctx = CreateOptionAccounts::context(accounts)?;

        let is_2022 = accounts::check_token_program_key(ctx.accounts.token_2022)?;
        accounts::check_system_program_key(ctx.accounts.system_program)?;
        accounts::check_associated_token_program_key(ctx.accounts.associated)?;

        let extra_accounts = ctx.remaining_accounts.to_vec();

        let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

        let mint_data = ctx.accounts.token_mint.data.borrow();
        let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;

        let input_amount = if let Ok(transfer_fee_config) =
            mint.get_extension::<spl_token_2022::extension::transfer_fee::TransferFeeConfig>()
        {
            let fee_1 = transfer_fee_config
                .calculate_epoch_fee(Clock::get()?.epoch, args.token_amount)
                .ok_or(ProgramError::InvalidArgument)?;
            args.token_amount.saturating_sub(fee_1)
        } else {
            args.token_amount
        };

        msg!("Input post fees {} -> {}", args.token_amount, input_amount);

        let whole_tokens = (input_amount as f64) / f64::powf(10.0, mint.base.decimals as f64);
        let strike_price = (args.strike as f64) / (LAMPORTS_PER_SOL as f64);
        let total_lamports = ((args.strike as f64) * whole_tokens) as u64;

        let price_sol = (args.price as f64) / (LAMPORTS_PER_SOL as f64);

        let _create_cpi = CreateV1CpiBuilder::new(ctx.accounts.core_program)
            .authority(Some(ctx.accounts.program_pda))
            .asset(ctx.accounts.asset_account)
            .collection(Some(ctx.accounts.collection_account))
            .payer(ctx.accounts.user)
            .owner(Some(ctx.accounts.program_pda))
            .data_state(mpl_core::types::DataState::AccountState)
            .name(args.name.to_string())
            .uri(args.uri.to_string())
            .plugins(vec![
                mpl_core::types::PluginAuthorityPair {
                    plugin: mpl_core::types::Plugin::PermanentBurnDelegate(
                        mpl_core::types::PermanentBurnDelegate {},
                    ),
                    authority: Some(mpl_core::types::PluginAuthority::Address {
                        address: *ctx.accounts.program_pda.key,
                    }),
                },
                mpl_core::types::PluginAuthorityPair {
                    plugin: mpl_core::types::Plugin::PermanentTransferDelegate(
                        mpl_core::types::PermanentTransferDelegate {},
                    ),
                    authority: Some(mpl_core::types::PluginAuthority::Address {
                        address: *ctx.accounts.program_pda.key,
                    }),
                },
            ])
            .system_program(ctx.accounts.system_program)
            .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
            .unwrap();

        mpl_core::instructions::AddPluginV1CpiBuilder::new(ctx.accounts.core_program)
            .asset(ctx.accounts.asset_account)
            .collection(Some(ctx.accounts.collection_account))
            .payer(ctx.accounts.user)
            .authority(Some(ctx.accounts.program_pda))
            .plugin(mpl_core::types::Plugin::Attributes(
                mpl_core::types::Attributes {
                    attribute_list: vec![
                        mpl_core::types::Attribute {
                            key: "creator".to_string(),
                            value: ctx.accounts.user.key.to_string(),
                        },
                        mpl_core::types::Attribute {
                            key: "side".to_string(),
                            value: args.side.to_string(),
                        },
                        mpl_core::types::Attribute {
                            key: "token_mint".to_string(),
                            value: ctx.accounts.token_mint.key.to_string(),
                        },
                        mpl_core::types::Attribute {
                            key: "token_amount".to_string(),
                            value: whole_tokens.to_string(),
                        },
                        mpl_core::types::Attribute {
                            key: "strike_price".to_string(),
                            value: strike_price.to_string(),
                        },
                        mpl_core::types::Attribute {
                            key: "end_time".to_string(),
                            value: args.end_date.to_string(),
                        },
                        mpl_core::types::Attribute {
                            key: "listed".to_string(),
                            value: 1.to_string(),
                        },
                        mpl_core::types::Attribute {
                            key: "option_price".to_string(),
                            value: price_sol.to_string(),
                        },
                        mpl_core::types::Attribute {
                            key: "seller".to_string(),
                            value: ctx.accounts.user.key.to_string(),
                        },
                        mpl_core::types::Attribute {
                            key: "num_sales".to_string(),
                            value: 0.to_string(),
                        },
                    ],
                },
            ))
            .system_program(ctx.accounts.system_program)
            .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
            .unwrap();

        // create the token accounts if required
        if **ctx.accounts.user_token_account.try_borrow_lamports()? == 0 {
            let create_user_ata_idx = create_associated_token_account(
                &ctx.accounts.user.key,
                &ctx.accounts.user.key,
                &ctx.accounts.token_mint.key,
                ctx.accounts.token_2022.key,
            );

            invoke(
                &create_user_ata_idx,
                &[
                    ctx.accounts.user.clone(),
                    ctx.accounts.token_mint.clone(),
                    ctx.accounts.user_token_account.clone(),
                    ctx.accounts.token_2022.clone(),
                ],
            )?;
        }

        if **ctx.accounts.program_token_account.try_borrow_lamports()? == 0 {
            let create_program_ata_idx = create_associated_token_account(
                &ctx.accounts.user.key,
                &ctx.accounts.program_pda.key,
                &ctx.accounts.token_mint.key,
                ctx.accounts.token_2022.key,
            );

            invoke(
                &create_program_ata_idx,
                &[
                    ctx.accounts.user.clone(),
                    ctx.accounts.program_pda.clone(),
                    ctx.accounts.token_mint.clone(),
                    ctx.accounts.program_token_account.clone(),
                    ctx.accounts.token_2022.clone(),
                ],
            )?;
        }

        // if this is an option for someone to buy tokens from the user, transfer them to the program now
        if args.side == 0 {
            utils::transfer_tokens(
                is_2022,
                args.token_amount,
                ctx.accounts.user_token_account,
                ctx.accounts.token_mint,
                ctx.accounts.program_token_account,
                ctx.accounts.user,
                ctx.accounts.token_2022,
                pda_bump_seed,
                vec![b"pda"],
                mint.base.decimals,
                &extra_accounts,
            )
            .unwrap();
        }
        // otherwise transfer the SOL to the PDA
        else {
            invoke(
                &system_instruction::transfer(
                    ctx.accounts.user.key,
                    ctx.accounts.program_pda.key,
                    total_lamports,
                ),
                &[ctx.accounts.user.clone(), ctx.accounts.program_pda.clone()],
            )?;
        }

        Ok(())
    }

    fn list<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        args: ListOptionAssetMeta,
    ) -> ProgramResult {
        let ctx = ListAccounts::context(accounts)?;

        let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

        let asset = mpl_core::Asset::from_bytes(&ctx.accounts.asset_account.data.borrow()[..])?;

        let mut asset_attributes = asset
            .plugin_list
            .attributes
            .unwrap()
            .attributes
            .attribute_list;

        asset_attributes[6].value = 1.to_string();
        let price_sol = (args.price as f64) / (LAMPORTS_PER_SOL as f64);
        asset_attributes[7].value = price_sol.to_string();
        asset_attributes[8].value = ctx.accounts.user.key.to_string();

        mpl_core::instructions::UpdatePluginV1CpiBuilder::new(ctx.accounts.core_program)
            .asset(ctx.accounts.asset_account)
            .collection(Some(ctx.accounts.collection_account))
            .payer(ctx.accounts.user)
            .authority(Some(ctx.accounts.program_pda))
            .plugin(mpl_core::types::Plugin::Attributes(
                mpl_core::types::Attributes {
                    attribute_list: asset_attributes,
                },
            ))
            .system_program(ctx.accounts.system_program)
            .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
            .unwrap();

        mpl_core::instructions::TransferV1CpiBuilder::new(ctx.accounts.core_program)
            .asset(ctx.accounts.asset_account)
            .authority(Some(ctx.accounts.user))
            .payer(ctx.accounts.user)
            .new_owner(ctx.accounts.program_pda)
            .collection(Some(ctx.accounts.collection_account))
            .system_program(Some(ctx.accounts.system_program))
            .invoke()
            .unwrap();

        Ok(())
    }

    pub fn execute<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<(), ProgramError> {
        let ctx = ExecuteAccounts::context(accounts)?;

        let extra_accounts = ctx.remaining_accounts.to_vec();

        let is_2022 = accounts::check_token_program_key(ctx.accounts.token_2022)?;
        accounts::check_system_program_key(ctx.accounts.system_program)?;
        accounts::check_associated_token_program_key(ctx.accounts.associated)?;

        let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

        let mint_data = ctx.accounts.token_mint.data.borrow();
        let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;

        let asset = mpl_core::Asset::from_bytes(&ctx.accounts.asset_account.data.borrow()[..])?;

        let asset_attributes = asset
            .plugin_list
            .attributes
            .unwrap()
            .attributes
            .attribute_list;

        let deadline: i64 = asset_attributes[5].value.parse::<i64>().unwrap();

        let clock = Clock::get()?;

        // can only claim tickets if it is after launch has ended
        if clock.unix_timestamp > (deadline / 1000) as i64 {
            msg!("Cannot execute option after deadline");
            return Err(OptionsError::PastDeadline.into());
        }

        // create the token accounts if required
        if **ctx.accounts.user_token_account.try_borrow_lamports()? == 0 {
            let create_user_ata_idx = create_associated_token_account(
                &ctx.accounts.user.key,
                &ctx.accounts.user.key,
                &ctx.accounts.token_mint.key,
                ctx.accounts.token_2022.key,
            );

            invoke(
                &create_user_ata_idx,
                &[
                    ctx.accounts.user.clone(),
                    ctx.accounts.token_mint.clone(),
                    ctx.accounts.user_token_account.clone(),
                    ctx.accounts.token_2022.clone(),
                ],
            )?;
        }

        let side: u8 = asset_attributes[1].value.parse::<u8>().unwrap();
        let whole_token_amount: f64 = asset_attributes[3].value.parse::<f64>().unwrap();
        let token_amount: u64 =
            (whole_token_amount * ((10 as u64).pow(mint.base.decimals as u32) as f64)) as u64;

        let strike: f64 = asset_attributes[4].value.parse::<f64>().unwrap();
        let lamports: u64 = (whole_token_amount * strike * (LAMPORTS_PER_SOL as f64)) as u64;

        // if this was a buy option then transfer the tokens to the user, and the SOL to the creator
        if side == 0 {
            utils::transfer_tokens(
                is_2022,
                token_amount,
                ctx.accounts.program_token_account,
                ctx.accounts.token_mint,
                ctx.accounts.user_token_account,
                ctx.accounts.program_pda,
                ctx.accounts.token_2022,
                pda_bump_seed,
                vec![b"pda"],
                mint.base.decimals,
                &extra_accounts,
            )
            .unwrap();

            invoke(
                &system_instruction::transfer(
                    ctx.accounts.user.key,
                    ctx.accounts.creator.key,
                    lamports,
                ),
                &[ctx.accounts.user.clone(), ctx.accounts.creator.clone()],
            )?;
        } else {
            invoke_signed(
                &system_instruction::transfer(
                    ctx.accounts.program_pda.key,
                    ctx.accounts.user.key,
                    lamports,
                ),
                &[ctx.accounts.user.clone(), ctx.accounts.program_pda.clone()],
                &[&[b"pda", &[pda_bump_seed]]],
            )?;

            utils::transfer_tokens(
                is_2022,
                token_amount,
                ctx.accounts.user_token_account,
                ctx.accounts.token_mint,
                ctx.accounts.creator_token_account,
                ctx.accounts.user,
                ctx.accounts.token_2022,
                pda_bump_seed,
                vec![b"pda"],
                mint.base.decimals,
                &extra_accounts,
            )
            .unwrap();
        }

        // finally burn the asset
        mpl_core::instructions::BurnV1CpiBuilder::new(ctx.accounts.core_program)
            .asset(ctx.accounts.asset_account)
            .collection(Some(ctx.accounts.collection_account))
            .authority(Some(ctx.accounts.program_pda))
            .payer(ctx.accounts.user)
            .system_program(Some(ctx.accounts.system_program))
            .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
            .unwrap();

        Ok(())
    }

    pub fn refund<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
    ) -> Result<(), ProgramError> {
        let ctx = RefundAccounts::context(accounts)?;

        let extra_accounts = ctx.remaining_accounts.to_vec();

        let is_2022 = accounts::check_token_program_key(ctx.accounts.token_2022)?;
        accounts::check_system_program_key(ctx.accounts.system_program)?;
        accounts::check_associated_token_program_key(ctx.accounts.associated)?;

        let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

        let mint_data = ctx.accounts.token_mint.data.borrow();
        let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&mint_data)?;

        let asset = mpl_core::Asset::from_bytes(&ctx.accounts.asset_account.data.borrow()[..])?;

        let asset_attributes = asset
            .plugin_list
            .attributes
            .unwrap()
            .attributes
            .attribute_list;

        let deadline: i64 = asset_attributes[5].value.parse::<i64>().unwrap();

        let clock = Clock::get()?;

        // can only claim tickets if it is after launch has ended
        if clock.unix_timestamp < (deadline / 1000) as i64 {
            msg!("Cannot refund option before deadline");
            return Err(OptionsError::BeforeDeadline.into());
        }

        let side: u8 = asset_attributes[1].value.parse::<u8>().unwrap();
        let whole_token_amount: f64 = asset_attributes[3].value.parse::<f64>().unwrap();
        let token_amount: u64 =
            (whole_token_amount * ((10 as u64).pow(mint.base.decimals as u32) as f64)) as u64;

        let strike: f64 = asset_attributes[4].value.parse::<f64>().unwrap();
        let lamports: u64 = (whole_token_amount * strike * (LAMPORTS_PER_SOL as f64)) as u64;

        // if this was a buy option then return the tokens to the creator
        if side == 0 {
            msg!("Refund buy option {}", token_amount);
            utils::transfer_tokens(
                is_2022,
                token_amount,
                ctx.accounts.program_token_account,
                ctx.accounts.token_mint,
                ctx.accounts.creator_token_account,
                ctx.accounts.program_pda,
                ctx.accounts.token_2022,
                pda_bump_seed,
                vec![b"pda"],
                mint.base.decimals,
                &extra_accounts,
            )
            .unwrap();
        } else {
            invoke_signed(
                &system_instruction::transfer(
                    ctx.accounts.program_pda.key,
                    ctx.accounts.creator.key,
                    lamports,
                ),
                &[ctx.accounts.user.clone(), ctx.accounts.program_pda.clone()],
                &[&[b"pda", &[pda_bump_seed]]],
            )?;
        }

        // finally transfer and burn the asset

        mpl_core::instructions::TransferV1CpiBuilder::new(ctx.accounts.core_program)
            .asset(ctx.accounts.asset_account)
            .collection(Some(ctx.accounts.collection_account))
            .authority(Some(ctx.accounts.program_pda))
            .payer(ctx.accounts.user)
            .new_owner(ctx.accounts.user)
            .system_program(Some(ctx.accounts.system_program))
            .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
            .unwrap();

        mpl_core::instructions::BurnV1CpiBuilder::new(ctx.accounts.core_program)
            .asset(ctx.accounts.asset_account)
            .collection(Some(ctx.accounts.collection_account))
            .authority(Some(ctx.accounts.user))
            .payer(ctx.accounts.user)
            .system_program(Some(ctx.accounts.system_program))
            .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
            .unwrap();

        Ok(())
    }

    fn purchase<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
        let ctx = PurchaseAccounts::context(accounts)?;

        let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

        let asset = mpl_core::Asset::from_bytes(&ctx.accounts.asset_account.data.borrow()[..])?;

        let mut asset_attributes = asset
            .plugin_list
            .attributes
            .unwrap()
            .attributes
            .attribute_list;

        // listed
        asset_attributes[6].value = 0.to_string();

        let current_purchases = asset_attributes[9].value.parse::<u32>().unwrap();
        asset_attributes[9].value = (current_purchases + 1).to_string();

        let lamports: u64 =
            (asset_attributes[7].value.parse::<f64>().unwrap() * (LAMPORTS_PER_SOL as f64)) as u64;

        msg!(
            "lamports: {} {}",
            lamports,
            asset_attributes[7].value.parse::<f64>().unwrap()
        );

        mpl_core::instructions::UpdatePluginV1CpiBuilder::new(ctx.accounts.core_program)
            .asset(ctx.accounts.asset_account)
            .collection(Some(ctx.accounts.collection_account))
            .payer(ctx.accounts.user)
            .authority(Some(ctx.accounts.program_pda))
            .plugin(mpl_core::types::Plugin::Attributes(
                mpl_core::types::Attributes {
                    attribute_list: asset_attributes,
                },
            ))
            .system_program(ctx.accounts.system_program)
            .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
            .unwrap();

        mpl_core::instructions::TransferV1CpiBuilder::new(ctx.accounts.core_program)
            .asset(ctx.accounts.asset_account)
            .authority(Some(ctx.accounts.program_pda))
            .payer(ctx.accounts.user)
            .new_owner(ctx.accounts.user)
            .collection(Some(ctx.accounts.collection_account))
            .system_program(Some(ctx.accounts.system_program))
            .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
            .unwrap();

        invoke(
            &system_instruction::transfer(ctx.accounts.user.key, ctx.accounts.seller.key, lamports),
            &[ctx.accounts.user.clone(), ctx.accounts.seller.clone()],
        )?;

        Ok(())
    }
}
