use core::time;

use crate::{
    accounts, amm_state,
    instruction::{
        accounts::{
            CreateCollectionAccounts, EnterLongAccounts, EnterShortAccounts, ExitLongAccounts,
            ExitShortAccounts, LiquidateLongAccounts, LiquidateShortAccounts,
        },
        EnterPositionArgs,
    },
    utils,
};
use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::next_account_info,
    clock::Clock,
    native_token::LAMPORTS_PER_SOL,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    system_instruction,
    sysvar::{rent, Sysvar},
};
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};
use solana_program::{log::sol_log_compute_units, serde_varint::serialize};

use mpl_core::instructions::{CreateCollectionV1CpiBuilder, CreateV1CpiBuilder};
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};

pub fn create_collection<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> ProgramResult {
    let ctx = CreateCollectionAccounts::context(accounts)?;

    let (_collection_account, bump_seed) = Pubkey::find_program_address(
        &[&ctx.accounts.amm.key.to_bytes(), b"Collection"],
        &program_id,
    );

    let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

    let _create_cpi = CreateCollectionV1CpiBuilder::new(ctx.accounts.core_program)
        .collection(ctx.accounts.collection_account)
        .payer(ctx.accounts.user)
        .update_authority(Some(ctx.accounts.pda))
        .name("Shorts".to_string())
        .uri("".to_string())
        .system_program(ctx.accounts.system_program)
        .invoke_signed(&[
            &[
                &ctx.accounts.amm.key.to_bytes(),
                b"Collection",
                &[bump_seed],
            ],
            &[b"pda", &[pda_bump_seed]],
        ])
        .unwrap();

    mpl_core::instructions::AddCollectionPluginV1CpiBuilder::new(ctx.accounts.core_program)
        .collection(ctx.accounts.collection_account)
        .payer(ctx.accounts.user)
        .authority(Some(ctx.accounts.pda))
        .plugin(mpl_core::types::Plugin::Attributes(
            mpl_core::types::Attributes {
                attribute_list: vec![mpl_core::types::Attribute {
                    key: "amm".to_string(),
                    value: ctx.accounts.amm.key.to_string(),
                }],
            },
        ))
        .system_program(ctx.accounts.system_program)
        .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
        .unwrap();

    Ok(())
}

pub fn buy_short<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: EnterPositionArgs,
) -> ProgramResult {
    msg!("in buy short");
    let ctx = EnterShortAccounts::context(accounts)?;

    let mut amm_seed_keys: Vec<Pubkey> = Vec::new();
    amm_state::get_amm_seeds(
        *ctx.accounts.base_mint.key,
        *ctx.accounts.quote_mint.key,
        &mut amm_seed_keys,
    );

    let amm_bump_seed = accounts::check_program_data_account(
        ctx.accounts.amm,
        program_id,
        vec![
            &amm_seed_keys[0].to_bytes(),
            &amm_seed_keys[1].to_bytes(),
            b"AMM",
        ],
    )
    .unwrap();

    let quote_2022 = accounts::check_token_program_key(ctx.accounts.quote_token_program)?;
    accounts::check_system_program_key(ctx.accounts.system_program)?;
    accounts::check_associated_token_program_key(ctx.accounts.associated_token)?;

    let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

    let mut amm_data = amm_state::AMM::try_from_slice(&ctx.accounts.amm.data.borrow()[..])?;

    let (base_amount, quote_amount, base_float_amount, base_decimals, quote_decimals) =
        utils::check_swap(
            amm_data.amm_base_amount,
            amm_data.amm_quote_amount,
            ctx.accounts.base_mint,
            ctx.accounts.quote_mint,
            args.borrow_amount,
            amm_data.fee,
            1,
            false,
        )?;

    if amm_data.short_base_amount - base_amount < 100 {
        msg!("Base quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    if amm_data.amm_quote_amount - quote_amount < 100 {
        msg!("Quote quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    // move the base tokens over from the short pool to the amm
    amm_data.short_base_amount -= base_amount;
    amm_data.amm_base_amount += base_amount;
    // and transfer the quote amount out, this will get recorded in the short core asset
    amm_data.amm_quote_amount -= quote_amount;

    // update the price with the current state
    let new_price = utils::get_price(
        amm_data.amm_base_amount,
        amm_data.amm_quote_amount,
        base_decimals,
        quote_decimals,
    )?;

    utils::update_price_account(
        ctx.accounts.user,
        ctx.accounts.price_data,
        new_price as f32,
        base_float_amount as f32,
    )?;

    amm_data.last_price = new_price as f32;
    amm_data.serialize(&mut &mut ctx.accounts.amm.data.borrow_mut()[..])?;

    // transfer the deposit
    if ctx.accounts.quote_mint.key == &accounts::wrapped_sol_mint_account::ID {
        msg!("transfer wrapped sol as sol");
        let ix = system_instruction::transfer(
            ctx.accounts.user.key,
            ctx.accounts.amm_quote.key,
            args.overhead,
        );
        invoke(
            &ix,
            &[ctx.accounts.user.clone(), ctx.accounts.amm_quote.clone()],
        )?;
        let sync_ix = spl_token::instruction::sync_native(
            ctx.accounts.quote_token_program.key,
            ctx.accounts.amm_quote.key,
        )?;
        invoke(
            &sync_ix,
            &[
                ctx.accounts.quote_token_program.clone(),
                ctx.accounts.amm_quote.clone(),
            ],
        )?;
    } else {
        utils::transfer_tokens(
            quote_2022,
            args.overhead,
            ctx.accounts.user_quote,
            ctx.accounts.quote_mint,
            ctx.accounts.amm_quote,
            ctx.accounts.user,
            ctx.accounts.quote_token_program,
            amm_bump_seed,
            vec![
                &amm_seed_keys[0].to_bytes(),
                &amm_seed_keys[1].to_bytes(),
                b"AMM",
            ],
            quote_decimals,
            &Vec::new(),
        )?;
    }

    // now setup the asset that will be used to track the short

    // check what was actually transferred for the deposit
    let deposit_amount = utils::check_transfer_fee(args.overhead, ctx.accounts.quote_mint)?;

    // the liquidation price will be the current price plus this deposit per token
    let liquidation_price = utils::get_price(
        base_amount,
        deposit_amount + quote_amount,
        base_decimals,
        quote_decimals,
    )?;

    let short_price = utils::get_price(base_amount, quote_amount, base_decimals, quote_decimals)?;

    msg!("base amount {}", base_amount);
    msg!("deposit_amount  {}", deposit_amount);
    msg!("price  {}", new_price);
    msg!("liquidation_price  {}", liquidation_price);

    let clock = Clock::get()?;
    let timestamp = clock.unix_timestamp;

    let _create_cpi = CreateV1CpiBuilder::new(ctx.accounts.core_program)
        .authority(Some(ctx.accounts.pda))
        .asset(ctx.accounts.asset_account)
        .collection(Some(ctx.accounts.collection_account))
        .payer(ctx.accounts.user)
        .owner(Some(ctx.accounts.user))
        .data_state(mpl_core::types::DataState::AccountState)
        .name("short".to_string())
        .uri("".to_string())
        .plugins(vec![
            mpl_core::types::PluginAuthorityPair {
                plugin: mpl_core::types::Plugin::PermanentBurnDelegate(
                    mpl_core::types::PermanentBurnDelegate {},
                ),
                authority: Some(mpl_core::types::PluginAuthority::Address {
                    address: *ctx.accounts.pda.key,
                }),
            },
            mpl_core::types::PluginAuthorityPair {
                plugin: mpl_core::types::Plugin::PermanentTransferDelegate(
                    mpl_core::types::PermanentTransferDelegate {},
                ),
                authority: Some(mpl_core::types::PluginAuthority::Address {
                    address: *ctx.accounts.pda.key,
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
        .authority(Some(ctx.accounts.pda))
        .plugin(mpl_core::types::Plugin::Attributes(
            mpl_core::types::Attributes {
                attribute_list: vec![
                    mpl_core::types::Attribute {
                        key: "creator".to_string(),
                        value: ctx.accounts.user.key.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "amm".to_string(),
                        value: ctx.accounts.amm.key.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "base_amount".to_string(),
                        value: base_amount.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "quote_amount".to_string(),
                        value: quote_amount.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "entry_price".to_string(),
                        value: short_price.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "deposit_amount".to_string(),
                        value: deposit_amount.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "liquidation_price".to_string(),
                        value: liquidation_price.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "start_time".to_string(),
                        value: timestamp.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "direction".to_string(),
                        value: "short".to_string(),
                    },
                ],
            },
        ))
        .system_program(ctx.accounts.system_program)
        .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
        .unwrap();

    Ok(())
}

pub fn sell_short<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let ctx = ExitShortAccounts::context(accounts)?;

    let mut amm_seed_keys: Vec<Pubkey> = Vec::new();
    amm_state::get_amm_seeds(
        *ctx.accounts.base_mint.key,
        *ctx.accounts.quote_mint.key,
        &mut amm_seed_keys,
    );

    let amm_bump_seed = accounts::check_program_data_account(
        ctx.accounts.amm,
        program_id,
        vec![
            &amm_seed_keys[0].to_bytes(),
            &amm_seed_keys[1].to_bytes(),
            b"AMM",
        ],
    )
    .unwrap();

    let quote_2022 = accounts::check_token_program_key(ctx.accounts.quote_token_program)?;
    accounts::check_system_program_key(ctx.accounts.system_program)?;
    accounts::check_associated_token_program_key(ctx.accounts.associated_token)?;

    let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

    let asset = mpl_core::Asset::from_bytes(&ctx.accounts.asset_account.data.borrow()[..])?;

    let asset_attributes = asset
        .plugin_list
        .attributes
        .unwrap()
        .attributes
        .attribute_list;

    let short_base_amount = asset_attributes[2].value.parse::<u64>().unwrap();
    let short_quote_amount = asset_attributes[3].value.parse::<u64>().unwrap();
    let deposit_amount = asset_attributes[5].value.parse::<u64>().unwrap();
    let liquidation_price = asset_attributes[6].value.parse::<f64>().unwrap();
    let start_time = asset_attributes[7].value.parse::<u64>().unwrap();

    let direction = asset_attributes[8].value.as_str();

    msg!("Have direction {}", direction);
    if direction != "short" {
        msg!("Direction is not short");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut amm_data = amm_state::AMM::try_from_slice(&ctx.accounts.amm.data.borrow()[..])?;

    // calculate the borrow fee
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp as u64;
    let borrow_time_years = ((current_time - start_time) as f64) / 60.0 / 60.0 / 24.0 / 365.0;
    let borrow_fee = ((short_quote_amount as f64)
        * borrow_time_years
        * ((amm_data.borrow_cost as f64) / 100.0 / 100.0)
        + 1.0) as u64;

    // we use check_out_swap so that we add back exactly what we took to the short_base account
    let (base_amount, quote_amount, trade_float_amount, base_decimals, quote_decimals) =
        utils::check_out_swap(
            amm_data.amm_base_amount,
            amm_data.amm_quote_amount,
            ctx.accounts.base_mint,
            ctx.accounts.quote_mint,
            short_base_amount,
            amm_data.fee,
            0,
            false,
        )?;

    msg!("base amount {}", base_amount);
    msg!("quote_amount {}", quote_amount);
    msg!(
        "short_quote + deposit  {}",
        short_quote_amount + deposit_amount
    );

    // move the base tokens over from the amm to the short pool
    amm_data.short_base_amount += base_amount;
    amm_data.amm_base_amount -= base_amount;

    // if we are over the liquidation threshold then simulate a limit order at the liquidation price
    if quote_amount >= short_quote_amount + deposit_amount {
        amm_data.amm_quote_amount += short_quote_amount + deposit_amount;

        let price_after_limit = utils::get_price(
            amm_data.amm_base_amount,
            amm_data.amm_quote_amount,
            base_decimals,
            quote_decimals,
        )?;
        let limit_price = utils::get_price(
            base_amount,
            short_quote_amount + deposit_amount,
            base_decimals,
            quote_decimals,
        )?;

        msg!(
            "price is over liquidation price, simulating limit order {} {} {}",
            limit_price,
            liquidation_price,
            price_after_limit
        );
    } else {
        // and transfer the quote amount back
        amm_data.amm_quote_amount += quote_amount;

        // next consider the borrow fee
        let final_fee = borrow_fee.min(short_quote_amount + deposit_amount - quote_amount);

        amm_data.amm_quote_amount += final_fee;
        // finally the user gets whatever is left
        let user_amount = short_quote_amount + deposit_amount - quote_amount - final_fee;

        if user_amount > 0 {
            utils::transfer_tokens(
                quote_2022,
                user_amount,
                ctx.accounts.amm_quote,
                ctx.accounts.quote_mint,
                ctx.accounts.user_quote,
                ctx.accounts.amm,
                ctx.accounts.quote_token_program,
                amm_bump_seed,
                vec![
                    &amm_seed_keys[0].to_bytes(),
                    &amm_seed_keys[1].to_bytes(),
                    b"AMM",
                ],
                quote_decimals,
                &Vec::new(),
            )?;
        }
    }

    // update the price with the current state
    let new_price = utils::get_price(
        amm_data.amm_base_amount,
        amm_data.amm_quote_amount,
        base_decimals,
        quote_decimals,
    )?;

    utils::update_price_account(
        ctx.accounts.user,
        ctx.accounts.price_data,
        new_price as f32,
        trade_float_amount as f32,
    )?;

    amm_data.last_price = new_price as f32;
    amm_data.serialize(&mut &mut ctx.accounts.amm.data.borrow_mut()[..])?;

    // finally burn the asset
    mpl_core::instructions::BurnV1CpiBuilder::new(ctx.accounts.core_program)
        .asset(ctx.accounts.asset_account)
        .collection(Some(ctx.accounts.collection_account))
        .authority(Some(ctx.accounts.user))
        .payer(ctx.accounts.user)
        .system_program(Some(ctx.accounts.system_program))
        .invoke()
        .unwrap();

    Ok(())
}

pub fn liquidate_short<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let ctx = LiquidateShortAccounts::context(accounts)?;

    let mut amm_seed_keys: Vec<Pubkey> = Vec::new();
    amm_state::get_amm_seeds(
        *ctx.accounts.base_mint.key,
        *ctx.accounts.quote_mint.key,
        &mut amm_seed_keys,
    );

    let amm_bump_seed = accounts::check_program_data_account(
        ctx.accounts.amm,
        program_id,
        vec![
            &amm_seed_keys[0].to_bytes(),
            &amm_seed_keys[1].to_bytes(),
            b"AMM",
        ],
    )
    .unwrap();

    let _quote_2022 = accounts::check_token_program_key(ctx.accounts.quote_token_program)?;
    accounts::check_system_program_key(ctx.accounts.system_program)?;
    accounts::check_associated_token_program_key(ctx.accounts.associated_token)?;

    let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

    let asset = mpl_core::Asset::from_bytes(&ctx.accounts.asset_account.data.borrow()[..])?;

    let asset_attributes = asset
        .plugin_list
        .attributes
        .unwrap()
        .attributes
        .attribute_list;

    let short_base_amount = asset_attributes[2].value.parse::<u64>().unwrap();
    let short_quote_amount = asset_attributes[3].value.parse::<u64>().unwrap();
    let deposit_amount = asset_attributes[5].value.parse::<u64>().unwrap();
    let liquidation_price = asset_attributes[6].value.parse::<f64>().unwrap();
    let direction = asset_attributes[8].value.as_str();

    msg!("Have direction {}", direction);
    if direction != "short" {
        msg!("Direction is not short");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut amm_data = amm_state::AMM::try_from_slice(&ctx.accounts.amm.data.borrow()[..])?;

    let (base_amount, quote_amount, trade_float_amount, base_decimals, quote_decimals) =
        utils::check_out_swap(
            amm_data.amm_base_amount,
            amm_data.amm_quote_amount,
            ctx.accounts.base_mint,
            ctx.accounts.quote_mint,
            short_base_amount,
            amm_data.fee,
            0,
            false,
        )?;

    msg!("base amount {}", base_amount);
    msg!("quote_amount {}", quote_amount);
    msg!(
        "short_quote + deposit  {}",
        short_quote_amount + deposit_amount
    );

    // if we are under the liquidation threshold then we should exit
    if quote_amount < short_quote_amount + deposit_amount {
        msg!("price is under liquidation price, exiting");
        return Ok(());
    }

    // move the base tokens over from the amm to the short pool
    amm_data.short_base_amount += base_amount;
    amm_data.amm_base_amount -= base_amount;

    amm_data.amm_quote_amount += short_quote_amount + deposit_amount;

    let price_after_limit = utils::get_price(
        amm_data.amm_base_amount,
        amm_data.amm_quote_amount,
        base_decimals,
        quote_decimals,
    )?;
    let limit_price = utils::get_price(
        base_amount,
        short_quote_amount + deposit_amount,
        base_decimals,
        quote_decimals,
    )?;

    msg!(
        "price is over liquidation price, simulating limit order {} {} {}",
        limit_price,
        liquidation_price,
        price_after_limit
    );

    // update the price with the current state
    let new_price = utils::get_price(
        amm_data.amm_base_amount,
        amm_data.amm_quote_amount,
        base_decimals,
        quote_decimals,
    )?;

    utils::update_price_account(
        ctx.accounts.user,
        ctx.accounts.price_data,
        new_price as f32,
        trade_float_amount as f32,
    )?;

    amm_data.last_price = new_price as f32;
    amm_data.serialize(&mut &mut ctx.accounts.amm.data.borrow_mut()[..])?;

    // finally transfer burn the asset

    mpl_core::instructions::TransferV1CpiBuilder::new(ctx.accounts.core_program)
        .asset(ctx.accounts.asset_account)
        .collection(Some(ctx.accounts.collection_account))
        .authority(Some(ctx.accounts.pda))
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
        .invoke()
        .unwrap();

    Ok(())
}

pub fn enter_long<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: EnterPositionArgs,
) -> ProgramResult {
    let ctx = EnterLongAccounts::context(accounts)?;

    let mut amm_seed_keys: Vec<Pubkey> = Vec::new();
    amm_state::get_amm_seeds(
        *ctx.accounts.base_mint.key,
        *ctx.accounts.quote_mint.key,
        &mut amm_seed_keys,
    );

    let amm_bump_seed = accounts::check_program_data_account(
        ctx.accounts.amm,
        program_id,
        vec![
            &amm_seed_keys[0].to_bytes(),
            &amm_seed_keys[1].to_bytes(),
            b"AMM",
        ],
    )
    .unwrap();

    let base_2022 = accounts::check_token_program_key(ctx.accounts.base_token_program)?;
    let quote_2022 = accounts::check_token_program_key(ctx.accounts.quote_token_program)?;
    accounts::check_system_program_key(ctx.accounts.system_program)?;
    accounts::check_associated_token_program_key(ctx.accounts.associated_token)?;

    let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

    let mut amm_data = amm_state::AMM::try_from_slice(&ctx.accounts.amm.data.borrow()[..])?;

    let (base_amount, quote_amount, base_float_amount, base_decimals, quote_decimals) =
        utils::check_out_swap(
            amm_data.amm_base_amount,
            amm_data.amm_quote_amount,
            ctx.accounts.base_mint,
            ctx.accounts.quote_mint,
            args.borrow_amount,
            amm_data.fee,
            0,
            false,
        )?;

    if amm_data.amm_base_amount - base_amount < 100 {
        msg!("Base quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    if amm_data.long_quote_amount - quote_amount < 100 {
        msg!("Quote quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    // move the quote tokens over from the long pool to the amm
    amm_data.long_quote_amount -= quote_amount;
    amm_data.amm_quote_amount += quote_amount;
    // and transfer the base amount out, this will get recorded in the long core asset
    amm_data.amm_base_amount -= base_amount;

    // update the price with the current state
    let new_price = utils::get_price(
        amm_data.amm_base_amount,
        amm_data.amm_quote_amount,
        base_decimals,
        quote_decimals,
    )?;

    utils::update_price_account(
        ctx.accounts.user,
        ctx.accounts.price_data,
        new_price as f32,
        base_float_amount as f32,
    )?;

    amm_data.last_price = new_price as f32;
    amm_data.serialize(&mut &mut ctx.accounts.amm.data.borrow_mut()[..])?;

    msg!("base mint {:?}", ctx.accounts.base_mint.key);
    // transfer the deposit
    if ctx.accounts.base_mint.key == &accounts::wrapped_sol_mint_account::ID {
        msg!("transfer wrapped sol as sol");
        let ix = system_instruction::transfer(
            ctx.accounts.user.key,
            ctx.accounts.amm_base.key,
            args.overhead,
        );
        invoke(
            &ix,
            &[ctx.accounts.user.clone(), ctx.accounts.amm_base.clone()],
        )?;
        let sync_ix = spl_token::instruction::sync_native(
            ctx.accounts.base_token_program.key,
            ctx.accounts.amm_base.key,
        )?;
        invoke(
            &sync_ix,
            &[
                ctx.accounts.base_token_program.clone(),
                ctx.accounts.amm_base.clone(),
            ],
        )?;
    } else {
        utils::transfer_tokens(
            base_2022,
            args.overhead,
            ctx.accounts.user_base,
            ctx.accounts.base_mint,
            ctx.accounts.amm_base,
            ctx.accounts.user,
            ctx.accounts.base_token_program,
            amm_bump_seed,
            vec![
                &amm_seed_keys[0].to_bytes(),
                &amm_seed_keys[1].to_bytes(),
                b"AMM",
            ],
            base_decimals,
            &Vec::new(),
        )?;
    }

    // now setup the asset that will be used to track the long

    // check what was actually transferred for the deposit
    let deposit_amount = utils::check_transfer_fee(args.overhead, ctx.accounts.base_mint)?;

    // the liquidation price will be the current price plus this deposit per token
    let liquidation_price = utils::get_price(
        base_amount + deposit_amount,
        quote_amount,
        base_decimals,
        quote_decimals,
    )?;

    let long_price = utils::get_price(base_amount, quote_amount, base_decimals, quote_decimals)?;

    msg!("base amount {}", base_amount);
    msg!("deposit_amount  {}", deposit_amount);
    msg!("price  {}", new_price);
    msg!("liquidation_price  {}", liquidation_price);

    let clock = Clock::get()?;
    let timestamp = clock.unix_timestamp;

    let _create_cpi = CreateV1CpiBuilder::new(ctx.accounts.core_program)
        .authority(Some(ctx.accounts.pda))
        .asset(ctx.accounts.asset_account)
        .collection(Some(ctx.accounts.collection_account))
        .payer(ctx.accounts.user)
        .owner(Some(ctx.accounts.user))
        .data_state(mpl_core::types::DataState::AccountState)
        .name("long".to_string())
        .uri("".to_string())
        .plugins(vec![
            mpl_core::types::PluginAuthorityPair {
                plugin: mpl_core::types::Plugin::PermanentBurnDelegate(
                    mpl_core::types::PermanentBurnDelegate {},
                ),
                authority: Some(mpl_core::types::PluginAuthority::Address {
                    address: *ctx.accounts.pda.key,
                }),
            },
            mpl_core::types::PluginAuthorityPair {
                plugin: mpl_core::types::Plugin::PermanentTransferDelegate(
                    mpl_core::types::PermanentTransferDelegate {},
                ),
                authority: Some(mpl_core::types::PluginAuthority::Address {
                    address: *ctx.accounts.pda.key,
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
        .authority(Some(ctx.accounts.pda))
        .plugin(mpl_core::types::Plugin::Attributes(
            mpl_core::types::Attributes {
                attribute_list: vec![
                    mpl_core::types::Attribute {
                        key: "creator".to_string(),
                        value: ctx.accounts.user.key.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "amm".to_string(),
                        value: ctx.accounts.amm.key.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "base_amount".to_string(),
                        value: base_amount.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "quote_amount".to_string(),
                        value: quote_amount.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "entry_price".to_string(),
                        value: long_price.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "deposit_amount".to_string(),
                        value: deposit_amount.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "liquidation_price".to_string(),
                        value: liquidation_price.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "start_time".to_string(),
                        value: timestamp.to_string(),
                    },
                    mpl_core::types::Attribute {
                        key: "direction".to_string(),
                        value: "long".to_string(),
                    },
                ],
            },
        ))
        .system_program(ctx.accounts.system_program)
        .invoke_signed(&[&[b"pda", &[pda_bump_seed]]])
        .unwrap();

    Ok(())
}

pub fn exit_long<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let ctx = ExitLongAccounts::context(accounts)?;

    let mut amm_seed_keys: Vec<Pubkey> = Vec::new();
    amm_state::get_amm_seeds(
        *ctx.accounts.base_mint.key,
        *ctx.accounts.quote_mint.key,
        &mut amm_seed_keys,
    );

    let amm_bump_seed = accounts::check_program_data_account(
        ctx.accounts.amm,
        program_id,
        vec![
            &amm_seed_keys[0].to_bytes(),
            &amm_seed_keys[1].to_bytes(),
            b"AMM",
        ],
    )
    .unwrap();

    let base_2022 = accounts::check_token_program_key(ctx.accounts.base_token_program)?;
    let quote_2022 = accounts::check_token_program_key(ctx.accounts.quote_token_program)?;
    accounts::check_system_program_key(ctx.accounts.system_program)?;
    accounts::check_associated_token_program_key(ctx.accounts.associated_token)?;

    let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

    let asset = mpl_core::Asset::from_bytes(&ctx.accounts.asset_account.data.borrow()[..])?;

    let asset_attributes = asset
        .plugin_list
        .attributes
        .unwrap()
        .attributes
        .attribute_list;

    let borrowed_base_amount = asset_attributes[2].value.parse::<u64>().unwrap();
    let borrowed_quote_amount = asset_attributes[3].value.parse::<u64>().unwrap();
    let deposit_amount = asset_attributes[5].value.parse::<u64>().unwrap();
    let liquidation_price = asset_attributes[6].value.parse::<f64>().unwrap();
    let start_time = asset_attributes[7].value.parse::<u64>().unwrap();
    let direction = asset_attributes[8].value.as_str();

    msg!("Have direction {}", direction);
    if direction != "long" {
        msg!("Direction is not long");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut amm_data = amm_state::AMM::try_from_slice(&ctx.accounts.amm.data.borrow()[..])?;

    // calculate the borrow fee
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp as u64;
    let borrow_time_years = ((current_time - start_time) as f64) / 60.0 / 60.0 / 24.0 / 365.0;
    let borrow_fee = ((borrowed_base_amount as f64)
        * borrow_time_years
        * ((amm_data.borrow_cost as f64) / 100.0 / 100.0)
        + 1.0) as u64;

    // we use check_out_swap so that we add back exactly what we took to the long_quote account
    let (base_amount, quote_amount, trade_float_amount, base_decimals, quote_decimals) =
        utils::check_out_swap(
            amm_data.amm_base_amount,
            amm_data.amm_quote_amount,
            ctx.accounts.base_mint,
            ctx.accounts.quote_mint,
            borrowed_quote_amount,
            amm_data.fee,
            1,
            false,
        )?;

    msg!("base amount {}", base_amount);
    msg!("quote_amount {}", quote_amount);
    msg!("base + deposit  {}", borrowed_base_amount + deposit_amount);

    // move the base tokens over from the amm to the short pool
    amm_data.long_quote_amount += quote_amount;
    amm_data.amm_quote_amount -= quote_amount;

    // if we are over the liquidation threshold then simulate a limit order at the liquidation price
    if base_amount >= borrowed_base_amount + deposit_amount {
        amm_data.amm_base_amount += borrowed_base_amount + deposit_amount;

        let price_after_limit = utils::get_price(
            amm_data.amm_base_amount,
            amm_data.amm_quote_amount,
            base_decimals,
            quote_decimals,
        )?;
        let limit_price = utils::get_price(
            borrowed_base_amount + deposit_amount,
            quote_amount,
            base_decimals,
            quote_decimals,
        )?;

        msg!(
            "price is over liquidation price, simulating limit order {} {} {}",
            limit_price,
            liquidation_price,
            price_after_limit
        );
    } else {
        // and transfer the base amount back
        amm_data.amm_base_amount += base_amount;

        // next consider the borrow fee
        let final_fee = borrow_fee.min(borrowed_base_amount + deposit_amount - base_amount);

        amm_data.amm_base_amount += final_fee;
        // finally the user gets whatever is left
        let user_amount = borrowed_base_amount + deposit_amount - base_amount - final_fee;

        if user_amount > 0 {
            utils::transfer_tokens(
                base_2022,
                user_amount,
                ctx.accounts.amm_base,
                ctx.accounts.base_mint,
                ctx.accounts.user_base,
                ctx.accounts.amm,
                ctx.accounts.base_token_program,
                amm_bump_seed,
                vec![
                    &amm_seed_keys[0].to_bytes(),
                    &amm_seed_keys[1].to_bytes(),
                    b"AMM",
                ],
                base_decimals,
                &Vec::new(),
            )?;
        }
    }

    // update the price with the current state
    let new_price = utils::get_price(
        amm_data.amm_base_amount,
        amm_data.amm_quote_amount,
        base_decimals,
        quote_decimals,
    )?;

    utils::update_price_account(
        ctx.accounts.user,
        ctx.accounts.price_data,
        new_price as f32,
        trade_float_amount as f32,
    )?;

    amm_data.last_price = new_price as f32;
    amm_data.serialize(&mut &mut ctx.accounts.amm.data.borrow_mut()[..])?;

    // finally burn the asset
    mpl_core::instructions::BurnV1CpiBuilder::new(ctx.accounts.core_program)
        .asset(ctx.accounts.asset_account)
        .collection(Some(ctx.accounts.collection_account))
        .authority(Some(ctx.accounts.user))
        .payer(ctx.accounts.user)
        .system_program(Some(ctx.accounts.system_program))
        .invoke()
        .unwrap();

    Ok(())
}

pub fn liquidate_long<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let ctx = LiquidateLongAccounts::context(accounts)?;

    let mut amm_seed_keys: Vec<Pubkey> = Vec::new();
    amm_state::get_amm_seeds(
        *ctx.accounts.base_mint.key,
        *ctx.accounts.quote_mint.key,
        &mut amm_seed_keys,
    );

    let amm_bump_seed = accounts::check_program_data_account(
        ctx.accounts.amm,
        program_id,
        vec![
            &amm_seed_keys[0].to_bytes(),
            &amm_seed_keys[1].to_bytes(),
            b"AMM",
        ],
    )
    .unwrap();

    let _quote_2022 = accounts::check_token_program_key(ctx.accounts.quote_token_program)?;
    accounts::check_system_program_key(ctx.accounts.system_program)?;
    accounts::check_associated_token_program_key(ctx.accounts.associated_token)?;

    let (_pda_account, pda_bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

    let asset = mpl_core::Asset::from_bytes(&ctx.accounts.asset_account.data.borrow()[..])?;

    let asset_attributes = asset
        .plugin_list
        .attributes
        .unwrap()
        .attributes
        .attribute_list;

    let borrowed_base_amount = asset_attributes[2].value.parse::<u64>().unwrap();
    let borrowed_quote_amount = asset_attributes[3].value.parse::<u64>().unwrap();
    let deposit_amount = asset_attributes[5].value.parse::<u64>().unwrap();
    let liquidation_price = asset_attributes[6].value.parse::<f64>().unwrap();
    let direction = asset_attributes[8].value.as_str();

    msg!("Have direction {}", direction);
    if direction != "long" {
        msg!("Direction is not long");
        return Err(ProgramError::InvalidAccountData);
    }

    let mut amm_data = amm_state::AMM::try_from_slice(&ctx.accounts.amm.data.borrow()[..])?;

    let (base_amount, quote_amount, trade_float_amount, base_decimals, quote_decimals) =
        utils::check_out_swap(
            amm_data.amm_base_amount,
            amm_data.amm_quote_amount,
            ctx.accounts.base_mint,
            ctx.accounts.quote_mint,
            borrowed_quote_amount,
            amm_data.fee,
            1,
            false,
        )?;

    msg!("base amount {}", base_amount);
    msg!("quote_amount {}", quote_amount);
    msg!("base + deposit  {}", borrowed_base_amount + deposit_amount);

    // if we are under the liquidation threshold then we should exit
    if base_amount < borrowed_base_amount + deposit_amount {
        msg!("price is under liquidation price, exiting");
        return Ok(());
    }

    // move the base tokens over from the amm to the short pool
    amm_data.long_quote_amount += quote_amount;
    amm_data.amm_quote_amount -= quote_amount;

    amm_data.amm_base_amount += borrowed_base_amount + deposit_amount;

    let price_after_limit = utils::get_price(
        amm_data.amm_base_amount,
        amm_data.amm_quote_amount,
        base_decimals,
        quote_decimals,
    )?;
    let limit_price = utils::get_price(
        borrowed_base_amount + deposit_amount,
        quote_amount,
        base_decimals,
        quote_decimals,
    )?;

    msg!(
        "price is over liquidation price, simulating limit order {} {} {}",
        limit_price,
        liquidation_price,
        price_after_limit
    );

    // update the price with the current state
    let new_price = utils::get_price(
        amm_data.amm_base_amount,
        amm_data.amm_quote_amount,
        base_decimals,
        quote_decimals,
    )?;

    utils::update_price_account(
        ctx.accounts.user,
        ctx.accounts.price_data,
        new_price as f32,
        trade_float_amount as f32,
    )?;

    amm_data.last_price = new_price as f32;
    amm_data.serialize(&mut &mut ctx.accounts.amm.data.borrow_mut()[..])?;

    // finally transfer burn the asset

    mpl_core::instructions::TransferV1CpiBuilder::new(ctx.accounts.core_program)
        .asset(ctx.accounts.asset_account)
        .collection(Some(ctx.accounts.collection_account))
        .authority(Some(ctx.accounts.pda))
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
        .invoke()
        .unwrap();

    Ok(())
}
