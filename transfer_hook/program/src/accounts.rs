use solana_program::pubkey::Pubkey;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
};
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022;

pub fn check_system_program_key<'a>(account_info: &'a AccountInfo<'a>) -> ProgramResult {
    if account_info.key != &solana_program::system_program::ID {
        msg!(
            "expected system program {}",
            solana_program::system_program::ID
        );
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(());
}

pub fn check_token_program_key<'a>(account_info: &'a AccountInfo<'a>) -> ProgramResult {
    if account_info.key != &spl_token::id() {
        msg!("expected token program {}", spl_token::id());
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(());
}

pub fn check_token_program_2022_key<'a>(account_info: &'a AccountInfo<'a>) -> ProgramResult {
    if account_info.key != &spl_token_2022::id() {
        msg!("expected token 2022 program {}", spl_token_2022::id());
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(());
}

pub fn check_associated_token_program_key<'a>(account_info: &'a AccountInfo<'a>) -> ProgramResult {
    if account_info.key != &spl_associated_token_account::ID {
        msg!(
            "expected associated token program {}",
            spl_associated_token_account::ID
        );
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(());
}

pub fn check_program_data_account<'a>(
    account_info: &'a AccountInfo<'a>,
    program_id: &Pubkey,
    seed: Vec<&[u8]>,
) -> Result<u8, ProgramError> {
    if seed.len() == 1 {
        let (expected_data_account, bump_seed) =
            Pubkey::find_program_address(&[seed[0]], &program_id);

        // the third account is the user's token account
        if account_info.key != &expected_data_account {
            msg!("expected program data account {}", expected_data_account);
            return Err(ProgramError::InvalidAccountData);
        }

        return Ok(bump_seed);
    }

    let (expected_data_account, bump_seed) =
        Pubkey::find_program_address(&[seed[0], seed[1]], &program_id);

    // the third account is the user's token account
    if account_info.key != &expected_data_account {
        msg!("expected program data account {}", expected_data_account);
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(bump_seed);
}

pub fn check_token_account<'a>(
    account_info: &'a AccountInfo<'a>,
    mint_account_info: &'a AccountInfo<'a>,
    token_account_info: &'a AccountInfo<'a>,
) -> ProgramResult {
    let expected_token_account = get_associated_token_address_with_program_id(
        &account_info.key,
        &mint_account_info.key,
        &spl_token_2022::id(),
    );

    // the third account is the user's token account
    if token_account_info.key != &expected_token_account {
        msg!(
            "expected token account {} for mint {} and account {}, recieved {}",
            expected_token_account,
            mint_account_info.key,
            account_info.key,
            token_account_info.key
        );
        return Err(ProgramError::InvalidAccountData);
    }

    return Ok(());
}
