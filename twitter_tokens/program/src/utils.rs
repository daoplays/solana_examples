use crate::state::{get_user_data_size, UserData};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack, pubkey::Pubkey, rent, clock::Clock, sysvar::Sysvar,
    program_error::ProgramError, native_token::LAMPORTS_PER_SOL
};
use borsh::{BorshDeserialize, BorshSerialize};
use spl_associated_token_account::instruction::create_associated_token_account;

pub fn create_program_account<'a>(
    funding_account: &AccountInfo<'a>,
    pda : &AccountInfo<'a>,
    program_id :  &Pubkey,
    bump_seed : u8

) -> ProgramResult
{

     // Check if the account has already been initialized
     if **pda.try_borrow_lamports()? > 0 {
        msg!("This account is already initialized. skipping");
        return Ok(());
    }

    msg!("Creating programs derived account");

    let data_size = 0;
    let space : u64 = data_size.try_into().unwrap();
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
    invoke_signed(
        &ix,
        &[funding_account.clone(), pda.clone()],
        &[&[b"token_account", &[bump_seed]]]
    )?;

    Ok(())
}


pub fn create_user_data_account<'a>(
    funding_account: &AccountInfo<'a>,
    data_account: &AccountInfo<'a>,
    program_id :  &Pubkey,
    bump_seed : u8,
    seed : &[u8],
    data_size : usize
) -> ProgramResult
{

    // Check if the account has already been initialized
    if **data_account.try_borrow_lamports()? > 0 {
        msg!("user's data account is already initialized. skipping");
        return Ok(());
    }

    msg!("Creating user's data account");
        
    // the bidders data account just holds a single usize giving their location in the
    // bid array and a bool
    let space : u64 = data_size.try_into().unwrap();
    let lamports = rent::Rent::default().minimum_balance(data_size);

    msg!("Require {} lamports for {} size data", lamports, data_size);
    let ix = solana_program::system_instruction::create_account(
        funding_account.key,
        data_account.key,
        lamports,
        space,
        program_id,
    );

    // Sign and submit transaction
    invoke_signed(
        &ix,
        &[funding_account.clone(), data_account.clone()],
        &[&[seed, &[bump_seed]]]
    )?;

    Ok(())
}


pub fn create_hashtag_data_account<'a>(
    funding_account: &AccountInfo<'a>,
    data_account: &AccountInfo<'a>,
    program_id :  &Pubkey,
    bump_seed : u8,
    seed_1 : &[u8],
    seed_2 : &[u8],
    seed_3 : &[u8],
    data_size : usize
) -> ProgramResult
{

    // Check if the account has already been initialized
    if **data_account.try_borrow_lamports()? > 0 {
        msg!("user's data account is already initialized. skipping");
        return Ok(());
    }

    msg!("Creating user's data account");
        
    // the bidders data account just holds a single usize giving their location in the
    // bid array and a bool
    let space : u64 = data_size.try_into().unwrap();
    let lamports = rent::Rent::default().minimum_balance(data_size);

    msg!("Require {} lamports for {} size data", lamports, data_size);
    let ix = solana_program::system_instruction::create_account(
        funding_account.key,
        data_account.key,
        lamports,
        space,
        program_id,
    );

    // Sign and submit transaction
    invoke_signed(
        &ix,
        &[funding_account.clone(), data_account.clone()],
        &[&[seed_1, seed_2, seed_3, &[bump_seed]]]
    )?;

    Ok(())
}


pub fn create_token_account<'a>(
    funding_account : &AccountInfo<'a>,
    wallet_account : &AccountInfo<'a>,
    token_mint_account : &AccountInfo<'a>,
    new_token_account : &AccountInfo<'a>,
    token_program_account : &AccountInfo<'a>

) -> ProgramResult
{
    if **new_token_account.try_borrow_lamports()? > 0 {
        msg!("Token account is already initialised.");
        return Ok(());

    }

    msg!("creating Token account");
    let create_ata_idx = create_associated_token_account(&funding_account.key, &wallet_account.key,&token_mint_account.key);

    invoke(
        &create_ata_idx,
        &[funding_account.clone(), new_token_account.clone(), wallet_account.clone(), token_mint_account.clone(), token_program_account.clone()],
    )?;

    Ok(())
}

pub fn transfer_tokens<'a>(
    amount : u64,
    token_source_account : &AccountInfo<'a>,
    token_dest_account : &AccountInfo<'a>,
    authority_account : &AccountInfo<'a>,
    token_program_account : &AccountInfo<'a>,
    bump_seed : u8

) -> ProgramResult
{
    let ix = spl_token::instruction::transfer(
        token_program_account.key,
        token_source_account.key,
        token_dest_account.key,
        authority_account.key,
        &[],
        amount,
    )?;

    invoke_signed(
        &ix,
        &[token_source_account.clone(), token_dest_account.clone(), authority_account.clone(), token_program_account.clone()],
        &[&[b"token_account", &[bump_seed]]]
    )?;

    Ok(())
}

pub fn to_sol(value : u64) -> f64 {
    (value as f64) / (LAMPORTS_PER_SOL as f64)
}
