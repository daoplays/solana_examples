use crate::state::{get_state_index, StateEnum, get_bid_status_size, TOKENS_WON, MAX_WINNERS, BID_BLOCK, N_BID_BLOCKS, BidValues, BidTimes};
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

pub fn get_bid_state(max_time : i64, program_data_account_info : &AccountInfo) ->  Result<(u16, u64), ProgramError> {


    // calculate the total bid amount and number of bidders at this time
    let mut total_bid : u64 = 0;
    let mut n_bidders : u16 = 0;
    for idx in 0..N_BID_BLOCKS {
        let bid_idx = get_state_index(StateEnum::BidAmounts {index: idx*BID_BLOCK});
        let time_idx = get_state_index(StateEnum::BidTimes {index: idx*BID_BLOCK});


        let bids = BidValues::try_from_slice(&program_data_account_info.data.borrow()[bid_idx.0..bid_idx.0 + BID_BLOCK*8])?; 
        let times = BidTimes::try_from_slice(&program_data_account_info.data.borrow()[time_idx.0..time_idx.0 + BID_BLOCK*8])?; 


        for jdx in 0..BID_BLOCK {
            if times.bid_times[jdx] < max_time && bids.bid_amounts[jdx] > 0 {
                total_bid += bids.bid_amounts[jdx];
                n_bidders += 1;
            }

        }
    }

    Ok((n_bidders, total_bid))
    
}

pub fn check_winners_state<'a>(
    n_bidders : u16, 
    program_data_account_info : &AccountInfo<'a>,
    program_token_account_info : &AccountInfo<'a>
) ->  Result<u8, ProgramError> {



    //msg!("n bidders : {}", n_bidders);
    // if there are no bidders then we have noone to choose
    if n_bidders == 0 {
        msg!("no bidders to be able to select winners");
        return Ok(0);
    }


    // if there aren't enough tokens available then we can't choose winners
    let min_tokens: u64 = TOKENS_WON;
    let program_token_account = spl_token::state::Account::unpack_unchecked(&program_token_account_info.try_borrow_data()?)?;

    let token_balance = program_token_account.amount;
    if token_balance < min_tokens {
        msg!("insufficient tokens in program account to select new winners: {} < {}", token_balance, min_tokens);
        return Ok(0);
    }

    let max_token_blocks = token_balance / TOKENS_WON;


    // set the number of winners to the max and check if we should decrease from there
    let mut n_winners = MAX_WINNERS as u8;

    // check if we have enough token blocks for this many
    if n_winners as u64 > max_token_blocks {
        n_winners = max_token_blocks as u8;
    }

    // finally check if we have enough bidders for this
    let max_winners_from_bidders = n_bidders / 64 + 1;
    if n_winners as u16 > max_winners_from_bidders {
        n_winners = max_winners_from_bidders as u8;
    }

    let prev_time_idx = get_state_index(StateEnum::PrevSelectionTime);
    let prev_time_selected = i64::try_from_slice(&program_data_account_info.data.borrow()[prev_time_idx.0..prev_time_idx.1])?;

    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    let time_passed = (current_time - prev_time_selected) as f64;

    
    // on average we expect a single bidder to wait 5 minutes before being selected
    // we therefore calculate time_per_bidder based on the number of bidders, and number of winners being selected
    // if this is below 3 seconds we just allow new winners to be selected so that there is less friction with large
    // numbers of bidders

    let time_per_bidder = (5.0 * 60.0) / ((n_bidders as f64) / (n_winners as f64));
    
    msg!("time_per_bidder {} time_passed: {} n_bidders {} token_balance {} max_blocks {}", time_per_bidder, time_passed, n_bidders, token_balance, max_token_blocks);

    if time_per_bidder > 3.0 && time_passed < time_per_bidder {
        return Ok(0);
    }

    msg!("Selecting {} new winners! ({} {})", n_winners, max_token_blocks, max_winners_from_bidders);

    
    Ok(n_winners)
}

pub fn update_bid_state<'a>(
    program_data_account_info : &AccountInfo<'a>
) -> ProgramResult {


    // calculate the total bid amount and number of bidders at this time
    let update = get_bid_state(i64::MAX, program_data_account_info)?;
    let n_bidders = update.0;
    let total_bid = update.1;

    // update number of bidders
    let n_bidders_idx = get_state_index(StateEnum::NBidders);
    n_bidders.serialize(&mut &mut program_data_account_info.data.borrow_mut()[n_bidders_idx.0..n_bidders_idx.1])?;

    // update total_bid_amount
    let total_bid_idx = get_state_index(StateEnum::TotalBidAmount);
    total_bid.serialize(&mut &mut program_data_account_info.data.borrow_mut()[total_bid_idx.0..total_bid_idx.1])?;
    
    Ok(())
}

pub fn to_sol(value : u64) -> f64 {
    (value as f64) / (LAMPORTS_PER_SOL as f64)
}