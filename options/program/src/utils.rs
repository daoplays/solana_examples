use mpl_core::instructions::TransferV1CpiBuilder;
use solana_program::{
    account_info::AccountInfo, instruction::AccountMeta, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey,
};

use crate::instruction::accounts::TransferTokenAccounts;

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
    let ctx = TransferTokenAccounts::context(accounts)?;

    let _transfer = TransferV1CpiBuilder::new(ctx.accounts.core_program)
        .asset(ctx.accounts.asset_account)
        .authority(Some(ctx.accounts.user))
        .payer(ctx.accounts.user)
        .new_owner(ctx.accounts.destination_account)
        .collection(Some(ctx.accounts.collection_account))
        .system_program(Some(ctx.accounts.system_program))
        .invoke()
        .unwrap();

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
    transfer_hook_accounts: &Vec<&AccountInfo<'a>>,
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
    invoke_signed(&ix, &account_infos, &[&[seed[0], &[bump_seed]]])?;

    Ok(())
}
