use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use mpl_core::instructions::TransferV1CpiBuilder;
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    instruction::AccountMeta,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent, system_instruction,
    sysvar::Sysvar,
};

use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::instruction as tokenInstruction;
use spl_token_2022::{
    extension::{BaseStateWithExtensions, StateWithExtensions},
    state::Account,
};

use crate::amm_state;

pub fn check_program_data_account<'a>(
    account_info: &'a AccountInfo<'a>,
    program_id: &Pubkey,
    seed: Vec<&[u8]>,
    name: String,
) -> Result<u8, ProgramError> {
    if seed.len() == 1 {
        let (expected_data_account, bump_seed) =
            Pubkey::find_program_address(&[seed[0]], &program_id);

        // the third account is the user's token account
        if account_info.key != &expected_data_account {
            msg!(
                "expected program data account {}  {}",
                name,
                expected_data_account
            );
            return Err(ProgramError::InvalidAccountData);
        }

        return Ok(bump_seed);
    }

    let (expected_data_account, bump_seed) =
        Pubkey::find_program_address(&[seed[0], seed[1]], &program_id);

    // the third account is the user's token account
    if account_info.key != &expected_data_account {
        msg!(
            "expected program data account {}  {}",
            name,
            expected_data_account
        );
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(bump_seed);
}

pub fn transfer_tokens<'a>(
    is_2022: bool,
    amount: u64,
    token_source_account: &AccountInfo<'a>,
    token_mint_account: &AccountInfo<'a>,
    token_dest_account: &AccountInfo<'a>,
    authority_account: &AccountInfo<'a>,
    token_program: &AccountInfo<'a>,
    bump_seed: u8,
    seed: Vec<&[u8]>,
    decimals: u8,
    transfer_hook_accounts: &Vec<AccountInfo<'a>>,
) -> Result<(), ProgramError> {
    if is_2022 {
        return transfer_T22_tokens(
            amount,
            token_source_account,
            token_mint_account,
            token_dest_account,
            authority_account,
            token_program,
            bump_seed,
            seed,
            decimals,
            transfer_hook_accounts,
        );
    } else {
        return transfer_spl_tokens(
            amount,
            token_source_account,
            token_dest_account,
            authority_account,
            token_program,
            bump_seed,
            seed,
        );
    }
}

pub fn transfer_spl_tokens<'a>(
    amount: u64,
    token_source_account: &AccountInfo<'a>,
    token_dest_account: &AccountInfo<'a>,
    authority_account: &AccountInfo<'a>,
    token_program_account: &AccountInfo<'a>,
    bump_seed: u8,
    seed: Vec<&[u8]>,
) -> Result<(), ProgramError> {
    let ix = spl_token::instruction::transfer(
        token_program_account.key,
        token_source_account.key,
        token_dest_account.key,
        authority_account.key,
        &[],
        amount,
    )?;

    // Sign and submit transaction
    if seed.len() == 1 {
        // Sign and submit transaction
        invoke_signed(
            &ix,
            &[
                token_source_account.clone(),
                token_dest_account.clone(),
                authority_account.clone(),
                token_program_account.clone(),
            ],
            &[&[seed[0], &[bump_seed]]],
        )?;
    }

    if seed.len() == 2 {
        // Sign and submit transaction
        invoke_signed(
            &ix,
            &[
                token_source_account.clone(),
                token_dest_account.clone(),
                authority_account.clone(),
                token_program_account.clone(),
            ],
            &[&[seed[0], seed[1], &[bump_seed]]],
        )?;
    }

    if seed.len() == 3 {
        // Sign and submit transaction
        invoke_signed(
            &ix,
            &[
                token_source_account.clone(),
                token_dest_account.clone(),
                authority_account.clone(),
                token_program_account.clone(),
            ],
            &[&[seed[0], seed[1], seed[2], &[bump_seed]]],
        )?;
    }

    Ok(())
}

pub fn transfer_T22_tokens<'a>(
    amount: u64,
    token_source_account: &AccountInfo<'a>,
    token_mint_account: &AccountInfo<'a>,
    token_dest_account: &AccountInfo<'a>,
    authority_account: &AccountInfo<'a>,
    token_program_2022_account: &AccountInfo<'a>,
    bump_seed: u8,
    seed: Vec<&[u8]>,
    decimals: u8,
    transfer_hook_accounts: &Vec<AccountInfo<'a>>,
) -> Result<(), ProgramError> {
    let mut ix = spl_token_2022::instruction::transfer_checked(
        token_program_2022_account.key,
        token_source_account.key,
        token_mint_account.key,
        token_dest_account.key,
        authority_account.key,
        &[],
        amount,
        decimals,
    )?;

    let mut account_infos = vec![
        token_source_account.clone(),
        token_mint_account.clone(),
        token_dest_account.clone(),
        authority_account.clone(),
        token_program_2022_account.clone(),
    ];

    if transfer_hook_accounts.len() > 0 {
        for i in 0..transfer_hook_accounts.len() {
            if transfer_hook_accounts[i].is_writable {
                ix.accounts.push(AccountMeta::new(
                    *transfer_hook_accounts[i].key,
                    transfer_hook_accounts[i].is_signer,
                ));
            } else {
                ix.accounts.push(AccountMeta::new_readonly(
                    *transfer_hook_accounts[i].key,
                    transfer_hook_accounts[i].is_signer,
                ));
            }
            account_infos.push(transfer_hook_accounts[i].clone());
        }
    }
    // Sign and submit transaction
    // Sign and submit transaction

    // Sign and submit transaction
    if seed.len() == 1 {
        // Sign and submit transaction
        invoke_signed(&ix, &account_infos, &[&[seed[0], &[bump_seed]]])?;
    }

    if seed.len() == 2 {
        // Sign and submit transaction
        invoke_signed(&ix, &account_infos, &[&[seed[0], seed[1], &[bump_seed]]])?;
    }

    if seed.len() == 3 {
        // Sign and submit transaction
        invoke_signed(
            &ix,
            &account_infos,
            &[&[seed[0], seed[1], seed[2], &[bump_seed]]],
        )?;
    }

    Ok(())
}

pub fn create_program_account<'a>(
    funding_account: &AccountInfo<'a>,
    pda: &AccountInfo<'a>,
    program_id: &Pubkey,
    bump_seed: u8,
    data_size: usize,
    seed: Vec<&[u8]>,
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
    if seed.len() == 1 {
        // Sign and submit transaction
        invoke_signed(
            &ix,
            &[funding_account.clone(), pda.clone()],
            &[&[seed[0], &[bump_seed]]],
        )?;
    }

    if seed.len() == 2 {
        // Sign and submit transaction
        invoke_signed(
            &ix,
            &[funding_account.clone(), pda.clone()],
            &[&[seed[0], seed[1], &[bump_seed]]],
        )?;
    }

    if seed.len() == 3 {
        // Sign and submit transaction
        invoke_signed(
            &ix,
            &[funding_account.clone(), pda.clone()],
            &[&[seed[0], seed[1], seed[2], &[bump_seed]]],
        )?;
    }

    Ok(())
}

pub fn get_token_balance<'a>(token_source_account: &AccountInfo<'a>) -> u64 {
    let base_data = &token_source_account.try_borrow_data().unwrap();
    let account_state = StateWithExtensions::<Account>::unpack(base_data).unwrap();

    return account_state.base.amount;
}

pub fn create_token_account<'a>(
    funding_account: &AccountInfo<'a>,
    wallet_account: &AccountInfo<'a>,
    token_mint_account: &AccountInfo<'a>,
    new_token_account: &AccountInfo<'a>,
    token_program_account: &AccountInfo<'a>,
) -> ProgramResult {
    if **new_token_account.try_borrow_lamports()? > 0 {
        msg!("Token account is already initialised.");
        return Ok(());
    }

    msg!("creating Token account");
    let create_ata_idx = create_associated_token_account(
        &funding_account.key,
        &wallet_account.key,
        &token_mint_account.key,
        &token_program_account.key,
    );

    invoke(
        &create_ata_idx,
        &[
            funding_account.clone(),
            new_token_account.clone(),
            wallet_account.clone(),
            token_mint_account.clone(),
            token_program_account.clone(),
        ],
    )?;

    Ok(())
}

pub fn transfer_wsol<'a>(
    amount: u64,
    fee_payer_account_info: &AccountInfo<'a>,
    user_account_info: &AccountInfo<'a>,
    temp_wsol_account_info: &AccountInfo<'a>,
    authority_account: &AccountInfo<'a>,
    source_wsol_account_info: &AccountInfo<'a>,
    wsol_mint_account_info: &AccountInfo<'a>,
    token_program_account_info: &AccountInfo<'a>,
    token_mint_pubkey: &Pubkey,
    pda_bump_seed: u8,
    pda_seeds: Vec<&[u8]>,
    temp_bump_seed: u8,
) -> ProgramResult {
    let token_lamports =
        rent::Rent::default().minimum_balance(spl_token::state::Account::LEN as usize);

    // create the temporary wsol account
    let base_ix = solana_program::system_instruction::create_account(
        fee_payer_account_info.key,
        temp_wsol_account_info.key,
        token_lamports,
        spl_token::state::Account::LEN as u64,
        token_program_account_info.key,
    );

    invoke_signed(
        &base_ix,
        &[
            fee_payer_account_info.clone(),
            temp_wsol_account_info.clone(),
            token_program_account_info.clone(),
        ],
        &[&[
            &fee_payer_account_info.key.to_bytes(),
            &token_mint_pubkey.to_bytes(),
            b"Temp",
            &[temp_bump_seed],
        ]],
    )?;

    let init_base_idx = tokenInstruction::initialize_account3(
        token_program_account_info.key,
        temp_wsol_account_info.key,
        wsol_mint_account_info.key,
        fee_payer_account_info.key,
    )
    .unwrap();

    invoke_signed(
        &init_base_idx,
        &[
            token_program_account_info.clone(),
            temp_wsol_account_info.clone(),
            wsol_mint_account_info.clone(),
            fee_payer_account_info.clone(),
        ],
        &[&[
            &fee_payer_account_info.key.to_bytes(),
            &token_mint_pubkey.to_bytes(),
            b"Temp",
            &[temp_bump_seed],
        ]],
    )?;

    self::transfer_spl_tokens(
        amount,
        source_wsol_account_info,
        temp_wsol_account_info,
        authority_account,
        token_program_account_info,
        pda_bump_seed,
        pda_seeds,
    )?;

    let close_idx = tokenInstruction::close_account(
        token_program_account_info.key,
        temp_wsol_account_info.key,
        user_account_info.key,
        fee_payer_account_info.key,
        &[],
    )?;

    invoke_signed(
        &close_idx,
        &[
            token_program_account_info.clone(),
            temp_wsol_account_info.clone(),
            user_account_info.clone(),
            fee_payer_account_info.clone(),
        ],
        &[&[
            &fee_payer_account_info.key.to_bytes(),
            &token_mint_pubkey.to_bytes(),
            b"Temp",
            &[temp_bump_seed],
        ]],
    )?;

    Ok(())
}

pub fn check_transfer_fee<'a>(
    quantity: u64,
    token_mint: &AccountInfo<'a>,
) -> Result<u64, ProgramError> {
    let token_mint_data = token_mint.data.borrow();
    let mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    let quantity_after_transfer = if let Ok(transfer_fee_config) =
        mint.get_extension::<spl_token_2022::extension::transfer_fee::TransferFeeConfig>()
    {
        let fee = transfer_fee_config
            .calculate_epoch_fee(Clock::get()?.epoch, quantity)
            .ok_or(ProgramError::InvalidArgument)?;
        quantity.saturating_sub(fee)
    } else {
        quantity
    };

    Ok(quantity_after_transfer)
}

pub fn check_out_swap<'a>(
    amm_base_amount: u64,
    amm_quote_amount: u64,
    base_token_mint: &AccountInfo<'a>,
    quote_token_mint: &AccountInfo<'a>,
    out_amount: u64,
    fee: u16,
    side: u8,
    is_transfer: bool,
) -> Result<(u64, u64, f64, u8, u8), ProgramError> {
    // see if we need to consider transfer fees
    let base_mint_data = base_token_mint.data.borrow();
    let base_mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&base_mint_data)?;
    let quote_mint_data = quote_token_mint.data.borrow();
    let quote_mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&quote_mint_data)?;

    let output = out_amount as f64;

    let in_amount = if side == 0 {
        output * (amm_quote_amount as f64) / ((amm_base_amount as f64) - output)
    } else {
        output * (amm_base_amount as f64) / ((amm_quote_amount as f64) - output)
    };
    let input_fees = 1.0 - (fee as f64) / 100.0 / 100.0;
    let input_post_fees = (in_amount / input_fees) as u64;

    let base_amount = if side == 0 {
        out_amount
    } else {
        input_post_fees
    };

    let quote_amount = if side == 0 {
        input_post_fees
    } else {
        out_amount
    };

    if input_post_fees == 0 {
        msg!("Quantisation has reduced input to zero");
        return Err(ProgramError::InvalidAccountData);
    }

    // check what is left over is enough
    if side == 0 && amm_base_amount - out_amount < 100 {
        msg!("Base quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    if side == 1 && amm_quote_amount - out_amount < 100 {
        msg!("Quote quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    let base_float_amount = base_amount as f64 / 10_f64.powi(base_mint.base.decimals as i32);

    Ok((
        base_amount,
        quote_amount,
        base_float_amount,
        base_mint.base.decimals,
        quote_mint.base.decimals,
    ))
}

pub fn check_swap<'a>(
    amm_base_amount: u64,
    amm_quote_amount: u64,
    base_token_mint: &AccountInfo<'a>,
    quote_token_mint: &AccountInfo<'a>,
    in_amount: u64,
    fee: u16,
    side: u8,
    is_transfer: bool,
) -> Result<(u64, u64, f64, u8, u8), ProgramError> {
    // see if we need to consider transfer fees
    let base_mint_data = base_token_mint.data.borrow();
    let base_mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&base_mint_data)?;
    let quote_mint_data = quote_token_mint.data.borrow();
    let quote_mint = StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&quote_mint_data)?;

    let input_amount_after_transfer = if let Ok(transfer_fee_config) = if side == 1 {
        base_mint.get_extension::<spl_token_2022::extension::transfer_fee::TransferFeeConfig>()
    } else {
        quote_mint.get_extension::<spl_token_2022::extension::transfer_fee::TransferFeeConfig>()
    } {
        let fee = transfer_fee_config
            .calculate_epoch_fee(Clock::get()?.epoch, in_amount)
            .ok_or(ProgramError::InvalidArgument)?;
        in_amount.saturating_sub(fee)
    } else {
        in_amount
    };

    let input_amount = if is_transfer {
        input_amount_after_transfer
    } else {
        in_amount
    };

    let input_fees = (input_amount as f64) * (fee as f64) / 100.0 / 100.0;
    let input_ex_fees = input_amount as f64 - input_fees;

    let output = if side == 0 {
        input_ex_fees * (amm_base_amount as f64) / ((amm_quote_amount as f64) + input_ex_fees)
    } else {
        input_ex_fees * (amm_quote_amount as f64) / (input_ex_fees + (amm_base_amount as f64))
    } as u64;

    let base_amount = if side == 0 { output } else { in_amount };

    let quote_amount = if side == 0 { in_amount } else { output };

    if output == 0 {
        msg!("Quantisation has reduced output to zero");
        return Err(ProgramError::InvalidAccountData);
    }

    // check what is left over is enough
    if side == 0 && amm_base_amount - output < 100 {
        msg!("Base quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    if side == 1 && amm_quote_amount - output < 100 {
        msg!("Quote quantity reduced below threshold");
        return Err(ProgramError::InvalidAccountData);
    }

    let base_float_amount = base_amount as f64 / 10_f64.powi(base_mint.base.decimals as i32);

    Ok((
        base_amount,
        quote_amount,
        base_float_amount,
        base_mint.base.decimals,
        quote_mint.base.decimals,
    ))
}

pub fn get_price(
    amm_base_amount: u64,
    amm_quote_amount: u64,
    base_decimals: u8,
    quote_decimals: u8,
) -> Result<f64, ProgramError> {
    let new_base_float_amount = amm_base_amount as f64 / 10_f64.powi(base_decimals as i32);
    let new_quote_float_amount = amm_quote_amount as f64 / 10_f64.powi(quote_decimals as i32);

    let new_price = new_quote_float_amount / new_base_float_amount;

    return Ok(new_price);
}

pub fn update_price_account<'a>(
    funding_account: &AccountInfo<'a>,
    price_data_account: &AccountInfo<'a>,
    price: f32,
    base_float_amount: f32,
) -> ProgramResult {
    let clock = Clock::get()?;

    let minute = clock.unix_timestamp / 60;
    let data_len = price_data_account.data_len();

    let candle_size = amm_state::get_candle_size();
    let mut n_candles = u32::try_from_slice(&price_data_account.data.borrow()[1..5])?;
    let mut last_candle = amm_state::OHLCV::try_from_slice(
        &price_data_account.data.borrow()[data_len - candle_size..data_len],
    )?;

    if minute == last_candle.timestamp {
        last_candle.low = f32::min(price, last_candle.low);
        last_candle.high = f32::max(price, last_candle.high);

        last_candle.close = price;
        last_candle.volume += base_float_amount;

        last_candle.serialize(
            &mut &mut price_data_account.data.borrow_mut()[data_len - candle_size..data_len],
        )?;
    } else {
        n_candles += 1;
        let new_candle: amm_state::OHLCV = amm_state::OHLCV {
            timestamp: minute,
            open: price,
            high: price,
            low: price,
            close: price,
            volume: base_float_amount,
        };

        msg!("adding candle {} {}", n_candles, new_candle.close);
        let new_len = data_len + candle_size;

        let old_lamports = rent::Rent::default().minimum_balance(data_len);
        let new_lamports = rent::Rent::default().minimum_balance(new_len);

        if new_lamports > old_lamports {
            msg!(
                "update price data account to new size: {} current_balance: {} new_balance {}",
                new_len,
                old_lamports,
                new_lamports
            );

            invoke(
                &system_instruction::transfer(
                    funding_account.key,
                    price_data_account.key,
                    new_lamports - old_lamports,
                ),
                &[funding_account.clone(), price_data_account.clone()],
            )?;
        }

        price_data_account.realloc(new_len, true)?;

        n_candles.serialize(&mut &mut price_data_account.data.borrow_mut()[1..5])?;
        new_candle.serialize(
            &mut &mut price_data_account.data.borrow_mut()[new_len - candle_size..new_len],
        )?;
    }

    Ok(())
}
