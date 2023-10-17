use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, native_token::LAMPORTS_PER_SOL,
    program::invoke, program::invoke_signed, program_error::ProgramError,
};
use spl_token_2022::instruction;
use solana_program::pubkey::Pubkey;

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


/// Seed for the state
pub const EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"extra-account-metas";

/// Get the state address PDA
pub fn get_extra_account_metas_address(mint: &Pubkey, program_id: &Pubkey) -> Pubkey {
    get_extra_account_metas_address_and_bump_seed(mint, program_id).0
}

/// Function used by programs implementing the interface, when creating the PDA,
/// to also get the bump seed
pub fn get_extra_account_metas_address_and_bump_seed(
    mint: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&collect_extra_account_metas_seeds(mint), program_id)
}

/// Function used by programs implementing the interface, when creating the PDA,
/// to get all of the PDA seeds
pub fn collect_extra_account_metas_seeds(mint: &Pubkey) -> [&[u8]; 2] {
    [EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()]
}

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
