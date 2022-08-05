use crate::state::{get_state_index, StateEnum, get_bid_status_size};
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack, pubkey::Pubkey, rent, clock::Clock, sysvar::Sysvar
};
use borsh::{BorshDeserialize, BorshSerialize};
use spl_associated_token_account::instruction::create_associated_token_account;

pub fn create_bidder_data_account<'a>(
    funding_account: &AccountInfo<'a>,
    data_account: &AccountInfo<'a>,
    program_id :  &Pubkey,
    bump_seed : u8
) -> ProgramResult
{

    // Check if the account has already been initialized
    if **data_account.try_borrow_lamports()? > 0 {
        msg!("bidder's data account is already initialized. skipping");
        return Ok(());
    }

    println!("Creating bidders data account");
        
    // the bidders data account just holds a single usize giving their location in the
    // bid array and a bool
    let data_size = get_bid_status_size();
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
        &[&[&funding_account.key.to_bytes(), &[bump_seed]]]
    )?;

    Ok(())
}

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

pub fn check_bid_state<'a>(
    program_data_account_info : &AccountInfo<'a>,
    program_token_account_info : &AccountInfo<'a>
) -> ProgramResult {


    // if there aren't enough tokens available then we can't choose winners
    let min_tokens: u64 = 100;
    let program_token_account = spl_token::state::Account::unpack_unchecked(&program_token_account_info.try_borrow_data()?)?;

    let token_balance = program_token_account.amount;
    if token_balance < min_tokens {
        msg!("insufficient tokens in program account to select new winners: {} < {}", token_balance, min_tokens);
        return Ok(());
    }

    // if there are no bidders then we have noone to choose
    let n_bidders_idx = get_state_index(StateEnum::NBidders);
    let n_bidders = u32::try_from_slice(&program_data_account_info.data.borrow()[n_bidders_idx.0..n_bidders_idx.1])?;

    if n_bidders == 0 {
        msg!("no bidders to be able to select winners");
        return Ok(());
    }

    let select_winners_idx = get_state_index(StateEnum::SelectWinners);
    let mut select_winners = bool::try_from_slice(&program_data_account_info.data.borrow()[select_winners_idx.0..select_winners_idx.1])?;

    // if we have already said we need to select winners there is nothing else to do
    if select_winners {
        msg!("already waiting to select winners {}", select_winners);
        return Ok(());
    }

    let prev_time_idx = get_state_index(StateEnum::PrevSelectionTime);
    let prev_time_selected = i64::try_from_slice(&program_data_account_info.data.borrow()[prev_time_idx.0..prev_time_idx.1])?;

    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    let time_passed = (current_time - prev_time_selected) as f64;

    msg!("time passed: {} n_bidders {}", time_passed, n_bidders);
    // the longer we have waited (up to a max of 5 minutes) the more likely it is that we will select winners
    // the more bidders we have (up to the max 1024) the more likely it is that we will select winners

    let n_bidders_frac: f64 = (n_bidders as f64) / 1024.0;
    let waiting_secs_frac: f64 = time_passed / (5.0 * 60.0);
 
    let total_frac = n_bidders_frac + waiting_secs_frac;

    if total_frac < 1.0 {
        return Ok(());
    }

    msg!("Selecting new winners!");
    select_winners = true;
    select_winners.serialize(&mut &mut program_data_account_info.data.borrow_mut()[select_winners_idx.0..select_winners_idx.1])?;
    
    Ok(())
}