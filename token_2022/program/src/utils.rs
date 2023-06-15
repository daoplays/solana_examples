use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, native_token::LAMPORTS_PER_SOL,
    program::invoke, program::invoke_signed, program_error::ProgramError,
};
use spl_token_2022::instruction;

use spl_associated_token_account::instruction::create_associated_token_account;

pub fn to_sol(value: u64) -> f64 {
    (value as f64) / (LAMPORTS_PER_SOL as f64)
}

pub fn to_lamports(value: f64) -> u64 {
    (value * LAMPORTS_PER_SOL as f64) as u64
}

pub fn create_token_account<'a>(
    funding_account: &AccountInfo<'a>,
    wallet_account: &AccountInfo<'a>,
    token_mint_account: &AccountInfo<'a>,
    new_token_account: &AccountInfo<'a>,
    token_program_account: &AccountInfo<'a>,
) -> Result<bool, ProgramError> {
    if **new_token_account.try_borrow_lamports()? > 0 {
        msg!("Token account is already initialised.");
        return Ok(false);
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

    Ok(true)
}

pub fn mint_tokens<'a>(
    user_token_account: &AccountInfo<'a>,
    token_mint_account: &AccountInfo<'a>,
    pda_account: &AccountInfo<'a>,
    token_program_account: &AccountInfo<'a>,
    quantity: u64,
    decimals: u8,
    pda_bump: u8,
    pda_seed: &[u8],
) -> ProgramResult {
    let mint_to_idx = instruction::mint_to_checked(
        token_program_account.key,
        token_mint_account.key,
        user_token_account.key,
        pda_account.key,
        &[pda_account.key],
        quantity,
        decimals,
    )
    .unwrap();

    invoke_signed(
        &mint_to_idx,
        &[
            token_program_account.clone(),
            token_mint_account.clone(),
            user_token_account.clone(),
            pda_account.clone(),
        ],
        &[&[pda_seed, &[pda_bump]]],
    )?;
    Ok(())
}
