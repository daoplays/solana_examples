use core::sync;

use borsh::{to_vec, BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg, program::invoke,
    program::invoke_signed, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    system_instruction, sysvar::rent, sysvar::Sysvar,
};

use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};

use crate::state;
use crate::utils;
use crate::{accounts, instruction::accounts::InitAMMAccounts};
use crate::{amm_state, instruction::SwapArgs};
use crate::{
    instruction::{
        accounts::{AddLiquidityAccounts, RemoveLiquidityAccounts, SwapAccounts},
        InitAMMArgs,
    },
    utils::update_price_account,
};

pub fn init_amm<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: InitAMMArgs,
) -> ProgramResult {
    msg!("in init_amm, getting accounts");
    let ctx: crate::instruction::accounts::Context<InitAMMAccounts> =
        InitAMMAccounts::context(accounts)?;

    if !ctx.accounts.user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let amm_account_lamports = **ctx.accounts.amm.try_borrow_lamports()?;

    if amm_account_lamports > 0 {
        msg!("amm already initialized");
        return Ok(());
    }

    let mut amm_seed_keys: Vec<Pubkey> = Vec::new();
    amm_state::get_amm_seeds(
        *ctx.accounts.base_token_mint.key,
        *ctx.accounts.quote_token_mint.key,
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

    let pda_bump_seed =
        accounts::check_program_data_account(ctx.accounts.pda, program_id, vec![b"pda"]).unwrap();

    accounts::check_token_account(
        ctx.accounts.amm,
        ctx.accounts.base_token_mint,
        ctx.accounts.amm_base,
        ctx.accounts.base_token_program,
    )?;

    accounts::check_token_account(
        ctx.accounts.amm,
        ctx.accounts.quote_token_mint,
        ctx.accounts.amm_quote,
        ctx.accounts.quote_token_program,
    )?;

    accounts::check_token_account(
        ctx.accounts.user,
        ctx.accounts.base_token_mint,
        ctx.accounts.user_base,
        ctx.accounts.base_token_program,
    )?;

    accounts::check_token_account(
        ctx.accounts.user,
        ctx.accounts.quote_token_mint,
        ctx.accounts.user_quote,
        ctx.accounts.quote_token_program,
    )?;

    accounts::check_token_account(
        ctx.accounts.user,
        ctx.accounts.lp_token_mint,
        ctx.accounts.user_lp,
        ctx.accounts.base_token_program,
    )?;

    let lp_bump_seed = accounts::check_program_data_account(
        ctx.accounts.lp_token_mint,
        program_id,
        vec![&ctx.accounts.amm.key.to_bytes(), b"LP"],
    )
    .unwrap();

    let num_price_accounts: u32 = 0;

    let price_data_bump_seed = accounts::check_program_data_account(
        ctx.accounts.price_data,
        program_id,
        vec![
            &ctx.accounts.amm.key.to_bytes(),
            &num_price_accounts.to_le_bytes(),
            b"TimeSeries",
        ],
    )
    .unwrap();

    accounts::check_system_program_key(ctx.accounts.system_program)?;
    let quote_2022 = accounts::check_token_program_key(ctx.accounts.quote_token_program)?;
    let base_2022 = accounts::check_token_program_key(ctx.accounts.base_token_program)?;
    accounts::check_associated_token_program_key(ctx.accounts.associated_token)?;

    // create the AMM data account
    utils::create_program_account(
        ctx.accounts.user,
        ctx.accounts.amm,
        program_id,
        amm_bump_seed,
        amm_state::get_amm_data_size(),
        vec![
            &amm_seed_keys[0].to_bytes(),
            &amm_seed_keys[1].to_bytes(),
            b"AMM",
        ],
    )?;

    // create the ATA for the base token
    if **ctx.accounts.amm_base.try_borrow_lamports()? == 0 {
        let create_base_ata_idx = create_associated_token_account(
            &ctx.accounts.user.key,
            &ctx.accounts.amm.key,
            &ctx.accounts.base_token_mint.key,
            ctx.accounts.base_token_program.key,
        );

        invoke(
            &create_base_ata_idx,
            &[
                ctx.accounts.user.clone(),
                ctx.accounts.amm.clone(),
                ctx.accounts.amm_base.clone(),
                ctx.accounts.base_token_mint.clone(),
                ctx.accounts.base_token_program.clone(),
            ],
        )?;
    }
    // create the ATA for the quote token
    if **ctx.accounts.amm_quote.try_borrow_lamports()? == 0 {
        let create_quote_ata_idx = create_associated_token_account(
            &ctx.accounts.user.key,
            &ctx.accounts.amm.key,
            &ctx.accounts.quote_token_mint.key,
            ctx.accounts.quote_token_program.key,
        );

        invoke(
            &create_quote_ata_idx,
            &[
                ctx.accounts.user.clone(),
                ctx.accounts.amm.clone(),
                ctx.accounts.amm_quote.clone(),
                ctx.accounts.quote_token_mint.clone(),
                ctx.accounts.quote_token_program.clone(),
            ],
        )?;
    }
    let mut amm_data = amm_state::AMM::try_from_slice(&ctx.accounts.amm.data.borrow()[..])?;

    let quote_amount = args.quote_quantity;
    let base_amount = args.base_quantity;

    if args.short_fraction > 100 {
        msg!("invalid short fraction {}", args.short_fraction);
        return Err(ProgramError::InvalidArgument);
    }

    let base_mint_data = ctx.accounts.base_token_mint.data.borrow();
    let base_mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&base_mint_data)?;
    let quote_mint_data = ctx.accounts.quote_token_mint.data.borrow();
    let quote_mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&quote_mint_data)?;

    msg!("{} {}", quote_amount, base_amount);

    msg!("transfer base token to AMM");
    utils::transfer_tokens(
        base_2022,
        args.base_quantity,
        ctx.accounts.user_base,
        ctx.accounts.base_token_mint,
        ctx.accounts.amm_base,
        ctx.accounts.user,
        ctx.accounts.base_token_program,
        pda_bump_seed,
        vec![b"pda"],
        base_mint.base.decimals,
        &Vec::new(),
    )?;

    if ctx.accounts.quote_token_mint.key == &accounts::wrapped_sol_mint_account::ID {
        msg!("transfer wrapped sol as sol");
        let ix = system_instruction::transfer(
            ctx.accounts.user.key,
            ctx.accounts.amm_quote.key,
            quote_amount,
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
            quote_amount,
            ctx.accounts.user_quote,
            ctx.accounts.quote_token_mint,
            ctx.accounts.amm_quote,
            ctx.accounts.user,
            ctx.accounts.quote_token_program,
            pda_bump_seed,
            vec![b"pda"],
            quote_mint.base.decimals,
            &Vec::new(),
        )?;
    }
    // see what actually arrived
    let initial_base_amount = utils::get_token_balance(ctx.accounts.amm_base);
    let initial_quote_amount = utils::get_token_balance(ctx.accounts.amm_quote);

    // divide the base amount into the short and amm pools
    let leverage_frac = (args.short_fraction as f64) / 100.0;

    let short_balance = ((initial_base_amount as f64) * leverage_frac) as u64;
    let amm_base_balance = initial_base_amount - short_balance;

    let long_balance = ((initial_quote_amount as f64) * leverage_frac) as u64;
    let amm_quote_balance = initial_quote_amount - long_balance;

    let base_float_amount = (amm_base_balance) as f64 / 10_f64.powi(base_mint.base.decimals as i32);
    let quote_float_amount =
        (amm_quote_balance) as f64 / 10_f64.powi(quote_mint.base.decimals as i32);

    let price = (quote_float_amount / base_float_amount) as f32;
    // liquidity is based on total across all pools
    let liquidity = f64::sqrt((initial_base_amount * initial_quote_amount) as f64) as u64;

    amm_data.account_type = state::AccountType::AMM;
    amm_data.base_mint = *ctx.accounts.base_token_mint.key;
    amm_data.quote_mint = *ctx.accounts.quote_token_mint.key;
    amm_data.lp_mint = *ctx.accounts.lp_token_mint.key;
    amm_data.base_key = *ctx.accounts.amm_base.key;
    amm_data.quote_key = *ctx.accounts.amm_quote.key;
    amm_data.fee = args.fee;
    amm_data.borrow_cost = args.borrow_fee;
    amm_data.leverage_frac = args.short_fraction;
    amm_data.amm_base_amount = amm_base_balance;
    amm_data.amm_quote_amount = amm_quote_balance;
    amm_data.short_base_amount = short_balance;
    amm_data.long_quote_amount = long_balance;

    amm_data.last_price = price;
    amm_data.lp_amount = liquidity;
    amm_data.serialize(&mut &mut ctx.accounts.amm.data.borrow_mut()[..])?;

    msg!(
        "initial balances {} {} {}",
        base_float_amount,
        quote_float_amount,
        price
    );

    let clock = Clock::get()?;
    let minute = clock.unix_timestamp / 60;

    let candle: amm_state::OHLCV = amm_state::OHLCV {
        timestamp: minute,
        open: price,
        high: price,
        low: price,
        close: price,
        volume: 0.0,
    };

    let time_series = amm_state::TimeSeriesData {
        account_type: state::AccountType::TimeSeries,
        data: vec![candle],
    };

    let price_data_size = to_vec(&time_series).unwrap().len();

    utils::create_program_account(
        ctx.accounts.user,
        ctx.accounts.price_data,
        program_id,
        price_data_bump_seed,
        price_data_size,
        vec![
            &ctx.accounts.amm.key.to_bytes(),
            &num_price_accounts.to_le_bytes(),
            b"TimeSeries",
        ],
    )?;

    time_series.serialize(&mut &mut ctx.accounts.price_data.data.borrow_mut()[..])?;

    // finally create the LP token

    if **ctx.accounts.lp_token_mint.try_borrow_lamports()? == 0 {
        let space = spl_token::state::Mint::LEN;
        // first create the mint account for the new NFT
        let mint_rent = rent::Rent::default().minimum_balance(space);

        let ix = solana_program::system_instruction::create_account(
            ctx.accounts.user.key,
            ctx.accounts.lp_token_mint.key,
            mint_rent,
            space as u64,
            ctx.accounts.base_token_program.key,
        );

        msg!("create account");
        // Sign and submit transaction
        invoke_signed(
            &ix,
            &[
                ctx.accounts.user.clone(),
                ctx.accounts.lp_token_mint.clone(),
            ],
            &[
                &[&ctx.accounts.amm.key.to_bytes(), b"LP", &[lp_bump_seed]],
                &[b"pda", &[pda_bump_seed]],
            ],
        )?;

        // initialize the mint, mint and freeze authority will be with the pda
        let mint_idx = spl_token_2022::instruction::initialize_mint2(
            ctx.accounts.base_token_program.key,
            ctx.accounts.lp_token_mint.key,
            ctx.accounts.pda.key,
            None,
            base_mint.base.decimals,
        )
        .unwrap();

        msg!("init mint");
        // Sign and submit transaction
        invoke_signed(
            &mint_idx,
            &[
                ctx.accounts.base_token_program.clone(),
                ctx.accounts.lp_token_mint.clone(),
                ctx.accounts.user.clone(),
            ],
            &[&[b"pda", &[pda_bump_seed]]],
        )?;
    }

    // mint the users lp tokens
    if **ctx.accounts.user_lp.try_borrow_lamports()? == 0 {
        // create the ATA for the quote token
        let create_lp_ata_idx = create_associated_token_account(
            &ctx.accounts.user.key,
            &ctx.accounts.user.key,
            &ctx.accounts.lp_token_mint.key,
            ctx.accounts.base_token_program.key,
        );

        invoke(
            &create_lp_ata_idx,
            &[
                ctx.accounts.user.clone(),
                ctx.accounts.user_lp.clone(),
                ctx.accounts.lp_token_mint.clone(),
                ctx.accounts.base_token_program.clone(),
            ],
        )?;
    }
    let mint_to_idx = spl_token_2022::instruction::mint_to_checked(
        ctx.accounts.base_token_program.key,
        ctx.accounts.lp_token_mint.key,
        ctx.accounts.user_lp.key,
        ctx.accounts.pda.key,
        &[ctx.accounts.pda.key],
        liquidity,
        base_mint.base.decimals,
    )
    .unwrap();

    invoke_signed(
        &mint_to_idx,
        &[
            ctx.accounts.base_token_program.clone(),
            ctx.accounts.lp_token_mint.clone(),
            ctx.accounts.user_lp.clone(),
            ctx.accounts.user.clone(),
            ctx.accounts.pda.clone(),
        ],
        &[&[b"pda", &[pda_bump_seed]]],
    )?;

    Ok(())
}

pub fn perform_swap<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: SwapArgs,
) -> ProgramResult {
    msg!("in perform_swap, getting accounts");

    let ctx: crate::instruction::accounts::Context<SwapAccounts> = SwapAccounts::context(accounts)?;
    if !ctx.accounts.user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut amm_data = amm_state::AMM::try_from_slice(&ctx.accounts.amm.data.borrow()[..])?;

    if *ctx.accounts.base_token_mint.key != amm_data.base_mint {
        msg!("Mint address account does not match");
        return Err(ProgramError::InvalidAccountData);
    }
    if *ctx.accounts.quote_token_mint.key != amm_data.quote_mint {
        msg!("Mint address account does not match");
        return Err(ProgramError::InvalidAccountData);
    }

    accounts::check_token_account(
        ctx.accounts.amm,
        ctx.accounts.base_token_mint,
        ctx.accounts.amm_base,
        ctx.accounts.base_token_program,
    )?;

    accounts::check_token_account(
        ctx.accounts.amm,
        ctx.accounts.quote_token_mint,
        ctx.accounts.amm_quote,
        ctx.accounts.quote_token_program,
    )?;

    accounts::check_token_account(
        ctx.accounts.user,
        ctx.accounts.base_token_mint,
        ctx.accounts.user_base,
        ctx.accounts.base_token_program,
    )?;

    accounts::check_token_account(
        ctx.accounts.user,
        ctx.accounts.quote_token_mint,
        ctx.accounts.user_quote,
        ctx.accounts.quote_token_program,
    )?;

    let mut amm_seed_keys: Vec<Pubkey> = Vec::new();
    amm_state::get_amm_seeds(
        *ctx.accounts.base_token_mint.key,
        *ctx.accounts.quote_token_mint.key,
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

    let pda_bump_seed =
        accounts::check_program_data_account(ctx.accounts.pda, program_id, vec![b"pda"]).unwrap();

    let base_2022 = accounts::check_token_program_key(ctx.accounts.base_token_program)?;
    let quote_2022 = accounts::check_token_program_key(ctx.accounts.quote_token_program)?;

    let (base_amount, quote_amount, base_float_amount, base_decimals, quote_decimals) =
        utils::check_swap(
            amm_data.amm_base_amount,
            amm_data.amm_quote_amount,
            ctx.accounts.base_token_mint,
            ctx.accounts.quote_token_mint,
            args.in_amount,
            amm_data.fee,
            args.side,
            true,
        )?;

    if args.side == 0 {
        if **ctx.accounts.user_base.try_borrow_lamports()? == 0 {
            let create_ata_idx = create_associated_token_account(
                &ctx.accounts.user.key,
                &ctx.accounts.user.key,
                &ctx.accounts.base_token_mint.key,
                ctx.accounts.base_token_program.key,
            );

            invoke(
                &create_ata_idx,
                &[
                    ctx.accounts.user.clone(),
                    ctx.accounts.user_base.clone(),
                    ctx.accounts.base_token_mint.clone(),
                    ctx.accounts.base_token_program.clone(),
                ],
            )?;
        }

        utils::transfer_tokens(
            quote_2022,
            args.in_amount,
            ctx.accounts.user_quote,
            ctx.accounts.quote_token_mint,
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

        utils::transfer_tokens(
            base_2022,
            base_amount,
            ctx.accounts.amm_base,
            ctx.accounts.base_token_mint,
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

        //update the amounts in the data
        amm_data.amm_base_amount -= base_amount;
        amm_data.amm_quote_amount += quote_amount;
    } else {
        utils::transfer_tokens(
            base_2022,
            args.in_amount,
            ctx.accounts.user_base,
            ctx.accounts.base_token_mint,
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

        utils::transfer_tokens(
            quote_2022,
            quote_amount,
            ctx.accounts.amm_quote,
            ctx.accounts.quote_token_mint,
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
        amm_data.amm_base_amount += base_amount;
        amm_data.amm_quote_amount -= quote_amount;
    }

    let new_price = utils::get_price(
        amm_data.amm_base_amount,
        amm_data.amm_quote_amount,
        base_decimals,
        quote_decimals,
    )?;

    update_price_account(
        ctx.accounts.user,
        ctx.accounts.price_data,
        new_price as f32,
        base_float_amount as f32,
    )?;

    amm_data.last_price = new_price as f32;
    amm_data.serialize(&mut &mut ctx.accounts.amm.data.borrow_mut()[..])?;

    return Ok(());
}

pub fn add_liquidity<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: SwapArgs,
) -> ProgramResult {
    let ctx: crate::instruction::accounts::Context<AddLiquidityAccounts> =
        AddLiquidityAccounts::context(accounts)?;

    if !ctx.accounts.user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut amm_seed_keys: Vec<Pubkey> = Vec::new();
    amm_state::get_amm_seeds(
        *ctx.accounts.base_token_mint.key,
        *ctx.accounts.quote_token_mint.key,
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

    /*     let amm_lamports = **ctx.accounts.amm.try_borrow_lamports()?;

        **ctx.accounts.amm.try_borrow_mut_lamports()? -= amm_lamports;
        **ctx.accounts.user.try_borrow_mut_lamports()? += amm_lamports;
        return Ok(());
    */
    let pda_bump_seed =
        accounts::check_program_data_account(ctx.accounts.pda, program_id, vec![b"pda"]).unwrap();

    let base_2022 = accounts::check_token_program_key(ctx.accounts.base_token_program)?;
    let quote_2022 = accounts::check_token_program_key(ctx.accounts.quote_token_program)?;

    let mut amm_data = amm_state::AMM::try_from_slice(&ctx.accounts.amm.data.borrow()[..])?;

    let amm_base_amount = amm_data.amm_base_amount + amm_data.short_base_amount;
    let amm_quote_amount = amm_data.amm_quote_amount + amm_data.long_quote_amount;

    // see if we need to consider transfer fees
    let base_mint_data = ctx.accounts.base_token_mint.data.borrow();
    let base_mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&base_mint_data)?;
    let quote_mint_data = ctx.accounts.quote_token_mint.data.borrow();
    let quote_mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&quote_mint_data)?;

    let base_added = if let Ok(transfer_fee_config) =
        base_mint.get_extension::<spl_token_2022::extension::transfer_fee::TransferFeeConfig>()
    {
        let fee = transfer_fee_config
            .calculate_epoch_fee(Clock::get()?.epoch, args.in_amount)
            .ok_or(ProgramError::InvalidArgument)?;
        args.in_amount.saturating_sub(fee)
    } else {
        args.in_amount
    };

    let quote_added = ((base_added as f64) * (amm_quote_amount as f64)
        / ((base_added as f64) + (amm_base_amount as f64))) as u64;

    if quote_added == 0 {
        msg!("Quantisation has reduced input to zero");
        return Err(ProgramError::InvalidAccountData);
    }

    let quote_to_send = if let Ok(transfer_fee_config) =
        quote_mint.get_extension::<spl_token_2022::extension::transfer_fee::TransferFeeConfig>()
    {
        let fee = transfer_fee_config
            .calculate_inverse_epoch_fee(Clock::get()?.epoch, quote_added)
            .ok_or(ProgramError::InvalidArgument)?;
        quote_added + fee
    } else {
        quote_added
    };

    let lp_generated =
        ((amm_data.lp_amount as f64) * (base_added as f64) / (amm_base_amount as f64)) as u64;

    msg!(
        "{} {} {} {}",
        amm_data.lp_amount,
        base_added,
        amm_base_amount,
        lp_generated
    );

    utils::transfer_tokens(
        base_2022,
        args.in_amount,
        ctx.accounts.user_base,
        ctx.accounts.base_token_mint,
        ctx.accounts.amm_base,
        ctx.accounts.user,
        ctx.accounts.base_token_program,
        amm_bump_seed,
        vec![
            &amm_seed_keys[0].to_bytes(),
            &amm_seed_keys[1].to_bytes(),
            b"AMM",
        ],
        base_mint.base.decimals,
        &Vec::new(),
    )?;

    utils::transfer_tokens(
        quote_2022,
        quote_to_send,
        ctx.accounts.user_quote,
        ctx.accounts.quote_token_mint,
        ctx.accounts.amm_quote,
        ctx.accounts.user,
        ctx.accounts.quote_token_program,
        pda_bump_seed,
        vec![b"pda"],
        quote_mint.base.decimals,
        &Vec::new(),
    )?;

    if **ctx.accounts.user_lp.try_borrow_lamports()? == 0 {
        let create_ata_idx = create_associated_token_account(
            &ctx.accounts.user.key,
            &ctx.accounts.user.key,
            &ctx.accounts.lp_token_mint.key,
            ctx.accounts.base_token_program.key,
        );

        invoke(
            &create_ata_idx,
            &[
                ctx.accounts.user.clone(),
                ctx.accounts.user_lp.clone(),
                ctx.accounts.lp_token_mint.clone(),
                ctx.accounts.base_token_program.clone(),
            ],
        )?;
    }

    let mint_to_idx = spl_token_2022::instruction::mint_to_checked(
        ctx.accounts.base_token_program.key,
        ctx.accounts.lp_token_mint.key,
        ctx.accounts.user_lp.key,
        ctx.accounts.pda.key,
        &[ctx.accounts.pda.key],
        lp_generated,
        base_mint.base.decimals,
    )
    .unwrap();

    invoke_signed(
        &mint_to_idx,
        &[
            ctx.accounts.base_token_program.clone(),
            ctx.accounts.lp_token_mint.clone(),
            ctx.accounts.user_lp.clone(),
            ctx.accounts.user.clone(),
            ctx.accounts.pda.clone(),
        ],
        &[&[b"pda", &[pda_bump_seed]]],
    )?;

    // divide the base amount into the short and amm pools
    let leverage_frac = (amm_data.leverage_frac as f64) / 100.0;

    let short_added = ((base_added as f64) * leverage_frac) as u64;
    let amm_base_added = base_added - short_added;

    let long_added = ((quote_added as f64) * leverage_frac) as u64;
    let amm_quote_added = quote_added - long_added;

    amm_data.amm_base_amount += amm_base_added;
    amm_data.amm_quote_amount += amm_quote_added;

    amm_data.short_base_amount += short_added;
    amm_data.long_quote_amount += long_added;

    amm_data.lp_amount += lp_generated;
    amm_data.serialize(&mut &mut ctx.accounts.amm.data.borrow_mut()[..])?;

    return Ok(());
}

pub fn remove_liquidity<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    args: SwapArgs,
) -> ProgramResult {
    let ctx: crate::instruction::accounts::Context<RemoveLiquidityAccounts> =
        RemoveLiquidityAccounts::context(accounts)?;

    if !ctx.accounts.user.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mut amm_seed_keys: Vec<Pubkey> = Vec::new();
    amm_state::get_amm_seeds(
        *ctx.accounts.base_token_mint.key,
        *ctx.accounts.quote_token_mint.key,
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

    let pda_bump_seed =
        accounts::check_program_data_account(ctx.accounts.pda, program_id, vec![b"pda"]).unwrap();

    let base_2022 = accounts::check_token_program_key(ctx.accounts.base_token_program)?;
    let quote_2022 = accounts::check_token_program_key(ctx.accounts.quote_token_program)?;

    let mut amm_data = amm_state::AMM::try_from_slice(&ctx.accounts.amm.data.borrow()[..])?;

    let total_base_amount = (amm_data.amm_base_amount + amm_data.short_base_amount) as f64;
    let total_quote_amount = (amm_data.amm_quote_amount + amm_data.long_quote_amount) as f64;
    let amm_lp_amount = amm_data.lp_amount as f64;

    let lp_to_remove = args.in_amount as f64;
    let base_to_remove = (total_base_amount * lp_to_remove / amm_lp_amount) as u64;
    let quote_to_remove = (total_quote_amount * lp_to_remove / amm_lp_amount) as u64;

    let leverage_frac = (amm_data.leverage_frac as f64) / 100.0;

    let short_removed = ((base_to_remove as f64) * leverage_frac) as u64;
    let amm_base_removed = base_to_remove - short_removed;

    let long_removed = ((quote_to_remove as f64) * leverage_frac) as u64;
    let amm_quote_removed = quote_to_remove - long_removed;

    if amm_data.amm_base_amount - amm_base_removed < 100 {
        msg!("Base quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    if amm_data.amm_quote_amount - amm_quote_removed < 100 {
        msg!("Quote quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    let base_mint_data = ctx.accounts.base_token_mint.data.borrow();
    let base_mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&base_mint_data)?;
    let quote_mint_data = ctx.accounts.quote_token_mint.data.borrow();
    let quote_mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&quote_mint_data)?;

    utils::transfer_tokens(
        base_2022,
        base_to_remove as u64,
        ctx.accounts.amm_base,
        ctx.accounts.base_token_mint,
        ctx.accounts.user_base,
        ctx.accounts.amm,
        ctx.accounts.base_token_program,
        amm_bump_seed,
        vec![
            &amm_seed_keys[0].to_bytes(),
            &amm_seed_keys[1].to_bytes(),
            b"AMM",
        ],
        base_mint.base.decimals,
        &Vec::new(),
    )?;

    utils::transfer_tokens(
        quote_2022,
        quote_to_remove as u64,
        ctx.accounts.amm_quote,
        ctx.accounts.quote_token_mint,
        ctx.accounts.user_quote,
        ctx.accounts.amm,
        ctx.accounts.quote_token_program,
        amm_bump_seed,
        vec![
            &amm_seed_keys[0].to_bytes(),
            &amm_seed_keys[1].to_bytes(),
            b"AMM",
        ],
        quote_mint.base.decimals,
        &Vec::new(),
    )?;

    let burn_instruction = spl_token_2022::instruction::burn(
        &ctx.accounts.base_token_program.key,
        &ctx.accounts.user_lp.key,
        &ctx.accounts.lp_token_mint.key,
        &ctx.accounts.user.key,
        &[],
        args.in_amount,
    )
    .unwrap();

    invoke(
        &burn_instruction,
        &[
            ctx.accounts.base_token_program.clone(),
            ctx.accounts.user_lp.clone(),
            ctx.accounts.lp_token_mint.clone(),
            ctx.accounts.user.clone(),
        ],
    )?;

    // check we arn't going to remove too much due to liquidity being lent out
    if amm_data.amm_base_amount - amm_base_removed < 100 {
        msg!("Base quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }
    if amm_data.amm_quote_amount - amm_quote_removed < 100 {
        msg!("Quote quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }
    if amm_data.short_base_amount - short_removed < 100 {
        msg!("Short quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }
    if amm_data.long_quote_amount - long_removed < 100 {
        msg!("Long quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    amm_data.amm_base_amount -= amm_base_removed;
    amm_data.amm_quote_amount -= amm_quote_removed;

    amm_data.short_base_amount -= short_removed;
    amm_data.long_quote_amount -= long_removed;

    amm_data.lp_amount -= args.in_amount;
    amm_data.serialize(&mut &mut ctx.accounts.amm.data.borrow_mut()[..])?;

    return Ok(());
}
