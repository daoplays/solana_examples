use mpl_core::instructions::TransferV1CpiBuilder;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    instruction::AccountMeta,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};

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

pub fn transfer<'a>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();
    let source_account_info = next_account_info(account_info_iter)?;
    let destination_account_info = next_account_info(account_info_iter)?;
    let asset_account_info: &AccountInfo<'a> = next_account_info(account_info_iter)?;
    let collection_account_info: &AccountInfo<'a> = next_account_info(account_info_iter)?;
    let core_program_account_info = next_account_info(account_info_iter)?;
    let system_program_account_info = next_account_info(account_info_iter)?;

    let _transfer = TransferV1CpiBuilder::new(core_program_account_info)
        .asset(asset_account_info)
        .authority(Some(source_account_info))
        .payer(source_account_info)
        .new_owner(destination_account_info)
        .collection(Some(collection_account_info))
        .system_program(Some(system_program_account_info))
        .invoke()
        .unwrap();

    Ok(())
}
