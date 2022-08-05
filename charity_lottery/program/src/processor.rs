use std::str::FromStr;
use crate::state::{StateEnum, get_state_index, Charity, CharityData, get_charity_size, BidderData, get_bid_status_size, BidValues};
use crate::instruction::{InitMeta};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::native_token::LAMPORTS_PER_SOL;
use crate::accounts;
use crate::utils;
use enum_map::{enum_map, EnumMap};
use crate::randoms;


use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program::{invoke},
    system_instruction,
    clock::Clock, sysvar::Sysvar
};
use spl_associated_token_account::get_associated_token_address;

use crate::{instruction::DaoPlaysInstruction, instruction::BidData};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        let instruction = DaoPlaysInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            DaoPlaysInstruction::CreateDataAccount {metadata} => {
                msg!("Instruction: CreateDataAccount");
                Self::create_data_account(accounts, program_id, metadata)
            },
            DaoPlaysInstruction::PlaceBid {bid_data} => {
                msg!("Instruction: PlaceBid");
                Self::process_place_bid(accounts, bid_data, program_id)
            },
            DaoPlaysInstruction::SelectWinners => {
                msg!("Instruction: SelectWinners");
                Self::select_winners(accounts, program_id)
            },
            DaoPlaysInstruction::SendTokens => {
                msg!("Instruction: SendTokens");
                Self::send_tokens(accounts, program_id)
            }
        }
    } 
 


    fn create_data_account(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : InitMeta
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed eight accounts, get them all first and then check their value is as expected
        let funding_account_info = next_account_info(account_info_iter)?;

        let program_derived_account_info = next_account_info(account_info_iter)?;
        let program_data_account_info = next_account_info(account_info_iter)?;

        let token_source_account_info = next_account_info(account_info_iter)?;
        let program_token_account_info = next_account_info(account_info_iter)?;
        let token_mint_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !funding_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // only we should be able to call this function
        if funding_account_info.key != &accounts::get_expected_daoplays_key() {
            msg!("expected first account to be a daoplays account  {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the second account is the program derived address which we can verify with find_program_address
        let (expected_pda_key, bump_seed) = accounts::get_expected_program_address_key(program_id);
         
        if program_derived_account_info.key != &expected_pda_key {
            msg!("expected second account to be PDA {}", expected_pda_key);
            return Err(ProgramError::InvalidAccountData);
        }

        // the next account is the programs data account
        if program_data_account_info.key != &accounts::get_expected_data_account_key(program_id) {
            msg!("expected second account to be program data account {}", accounts::get_expected_data_account_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }
  
        // the third account is the source of the tokens which we can verify with get_associated_token_address
        if token_source_account_info.key != &accounts::get_expected_daoplays_token_key() {
            msg!("expected third account to be the funder's token account {}", accounts::get_expected_daoplays_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the fourth account is the program's token account
        if program_token_account_info.key != &accounts::get_expected_program_token_key(program_id) {
            msg!("expected fourth account to be the program's token account {}", accounts::get_expected_program_token_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }

        // the fifth account is the mint address for the token
        if token_mint_account_info.key != &accounts::get_expected_token_mint_key() {
            msg!("expected fifth account to be the token's mint account {}", accounts::get_expected_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the ninth account is the token_program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected ninth account to be the token program {}", spl_token::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the tenth account is the associated_token_program
        if associated_token_account_info.key != &spl_associated_token_account::id() {
            msg!("expected tenth account to be the associated token program {}", spl_associated_token_account::id());
            return Err(ProgramError::InvalidAccountData);
        }
        
        // the eleventh and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eleventh account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }
        

        utils::create_program_account(
            funding_account_info,
            program_derived_account_info,
            program_id,
            bump_seed
        )?;
        
        utils::create_token_account(
            funding_account_info,
            program_derived_account_info,
            token_mint_account_info,
            program_token_account_info,
            token_program_account_info
        )?;

        utils::transfer_tokens(
            metadata.amount,
            token_source_account_info,
            program_token_account_info,
            funding_account_info,
            token_program_account_info,
            bump_seed
    
        )?;

        // now just initialise the prev_selected_time field of the state to clock now
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        let prev_time_idx = get_state_index(StateEnum::PrevSelectionTime);
        current_time.serialize(&mut &mut program_data_account_info.data.borrow_mut()[prev_time_idx.0..prev_time_idx.1])?;  

        let total_bid_idx = get_state_index(StateEnum::TotalBidAmount);
        0u64.serialize(&mut &mut program_data_account_info.data.borrow_mut()[total_bid_idx.0..total_bid_idx.1])?;  

        let n_bidders_idx = get_state_index(StateEnum::NBidders);
        0u32.serialize(&mut &mut program_data_account_info.data.borrow_mut()[n_bidders_idx.0..n_bidders_idx.1])?;  

        let bid_index_idx = get_state_index(StateEnum::BidIndex);
        0usize.serialize(&mut &mut program_data_account_info.data.borrow_mut()[bid_index_idx.0..bid_index_idx.1])?;


        for bid_index in 0..100 {
            let bid_idx = get_state_index(StateEnum::BidAmounts {index: bid_index});
            0u64.serialize(&mut &mut program_data_account_info.data.borrow_mut()[bid_idx.0..bid_idx.1])?; 

            let key_idx = get_state_index(StateEnum::BidKeys{index: bid_index});
            solana_program::system_program::id().serialize(&mut &mut program_data_account_info.data.borrow_mut()[key_idx.0..key_idx.1])?;
        }

        Ok(())


    }

    fn send_tokens(
        _accounts: &[AccountInfo],
        _program_id: &Pubkey,
    ) ->ProgramResult {

        msg!("in send_tokens");

    
        Ok(())
    }


    fn select_winners(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) ->ProgramResult {

        msg!("in select_winners");
        

        // we will use 3 streams, BTC,  ETH and SOL


        

        let account_info_iter = &mut accounts.iter();

        let funding_account_info = next_account_info(account_info_iter)?;


        // first accounts are the pyth oracles
        let btc_account_info = next_account_info(account_info_iter)?;
        let eth_account_info = next_account_info(account_info_iter)?;
        let sol_account_info = next_account_info(account_info_iter)?;

        let program_data_account_info = next_account_info(account_info_iter)?;


        // the first account should be the funding account and should be a signer
        if !funding_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // only we should be able to call this function
        if funding_account_info.key != &accounts::get_expected_daoplays_key() {
            msg!("expected first account to be a daoplays account  {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // check the accounts match what we expect
        if  btc_account_info.key != &accounts::get_expected_btc_key() || 
            eth_account_info.key != &accounts::get_expected_eth_key() ||
            sol_account_info.key != &accounts::get_expected_sol_key() 
        {
            return Err(ProgramError::InvalidAccountData);
        }

        // the next account is the programs data account
        if program_data_account_info.key != &accounts::get_expected_data_account_key(program_id) {
            msg!("expected second account to be program data account {}", accounts::get_expected_data_account_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }


        // first check we should actually be here
        let select_winners_idx = get_state_index(StateEnum::SelectWinners);
        let should_select = bool::try_from_slice(&program_data_account_info.data.borrow()[select_winners_idx.0..select_winners_idx.1])?;

        let n_bidders_idx = get_state_index(StateEnum::NBidders);
        let mut n_bidders = u32::try_from_slice(&program_data_account_info.data.borrow()[n_bidders_idx.0..n_bidders_idx.1])?;

        if !should_select || n_bidders == 0 {
            msg!("No need to select winners, exiting {} {}", should_select, n_bidders);
            return Ok(());
        }

        // generate the seed for selecting winners
        let mut pyth_random = randoms::generate_seed(
            btc_account_info,
            eth_account_info,
            sol_account_info
        );


        let n_winners: u8 = 10;// (n_bidders / 10 + 1) as u8;

        let mut ran_vec = vec![0.0f64; 128];
        for winner in 0..128 {
            pyth_random = randoms::shift_seed(pyth_random);
            let random_f64 =  randoms::generate_random(pyth_random);

            ran_vec[winner] = random_f64;

        }

        ran_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());


        msg!("bidders : {} winners : {}", n_bidders, n_winners);

        let n_winners_idx = get_state_index(StateEnum::NWinners);
        n_winners.serialize(&mut &mut program_data_account_info.data.borrow_mut()[n_winners_idx.0..n_winners_idx.1])?;  

        // get the total bid so we can scale our randoms
        let total_bid_idx = get_state_index(StateEnum::TotalBidAmount);
        let mut total_bid = u64::try_from_slice(&program_data_account_info.data.borrow()[total_bid_idx.0..total_bid_idx.1])?; 


        msg!("total: {}", (total_bid as f64) / (LAMPORTS_PER_SOL as f64));

        

        for winner in 0..n_winners {

            let mut cumulative_total : u64 = 0;
            let random_f64 = ran_vec[winner as usize];
            

            let threshold: u64 = ((total_bid as f64) * random_f64) as u64;

            msg!("Have total bid {} random {} threshold for winner {} of {}", (total_bid as f64)/(LAMPORTS_PER_SOL as f64), random_f64, (threshold as f64) / (LAMPORTS_PER_SOL as f64), winner);

            for idx in 0..8 {

                let bid_idx = get_state_index(StateEnum::BidAmounts {index: idx*128});
   
                let bids = BidValues::try_from_slice(&program_data_account_info.data.borrow()[bid_idx.0..bid_idx.0+256*8])?; 

                let mut found_winner = false;
                for bid_index in 0..256 {

                    let current_bid = bids.bid_amounts[bid_index];
                    
                    cumulative_total += current_bid;

                    if cumulative_total > threshold {

                        found_winner = true;
                        let total_index = idx * 128 + bid_index;
                        msg!("Have a winner: {} {}", total_index, (cumulative_total as f64) / (LAMPORTS_PER_SOL as f64));
    
                        // get the winners key from the program data account
                        let key_idx = get_state_index(StateEnum::BidKeys{index: bid_index});
                        let winners_key = Pubkey::try_from_slice(&program_data_account_info.data.borrow()[key_idx.0..key_idx.1])?; 

                        // and insert it into the winners array
                        let winner_idx = get_state_index(StateEnum::Winners{index: winner.try_into().unwrap()});
                        winners_key.serialize(&mut &mut program_data_account_info.data.borrow_mut()[winner_idx.0..winner_idx.1])?;
/*
                        // now clear the winners data in the program data account
                        // start by zero'ing their bid
                        // let bid_idx = get_state_index(StateEnum::BidAmounts {index: bid_index});
                        // 0u64.serialize(&mut &mut program_data_account_info.data.borrow_mut()[bid_idx.0..bid_idx.1])?;  

                        // and then clear their key
                        //solana_program::system_program::id().serialize(&mut &mut program_data_account_info.data.borrow_mut()[key_idx.0..key_idx.1])?;

                        // as a sanity check make sure current bid is less than total_bid
                        if current_bid > total_bid {
                            msg!("Current bid is greater than total bid, this shouldn't happen {} > {}", current_bid, total_bid);
                            return Ok(());
                        }

                        // finally decrement the number of bidders, and the total bid amount
                        n_bidders -= 1;
                        total_bid -= current_bid;

    */
                        break;


                    }

                    
                }
                if found_winner {
                    break;
                }

            }
        }

        return Ok(());
    /*

        // update n_bidders
        n_bidders.serialize(&mut &mut program_data_account_info.data.borrow_mut()[n_bidders_idx.0..n_bidders_idx.1])?;

        // update total_bid_amount
        total_bid.serialize(&mut &mut program_data_account_info.data.borrow_mut()[total_bid_idx.0..total_bid_idx.1])?;

        // update the select_winners bool to false again
        false.serialize(&mut &mut program_data_account_info.data.borrow_mut()[select_winners_idx.0..select_winners_idx.1])?;

        // update the prev_selected_time field of the state to clock now
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        let prev_time_idx = get_state_index(StateEnum::PrevSelectionTime);
        current_time.serialize(&mut &mut program_data_account_info.data.borrow_mut()[prev_time_idx.0..prev_time_idx.1])?;  

     */
    
        Ok(())
    }
    
    fn process_place_bid(
        accounts: &[AccountInfo],
        bid_data: BidData,
        program_id: &Pubkey,
    ) ->ProgramResult {

        let charity_key_map = enum_map!{
            Charity::UkraineERF  => "8bmmLYH2fJTUcLSz99Q1tP4xte9K41v3CeFJ6Qouogig",
            Charity::WaterOrg => "3aNSq2fKBypiiuPy4SgrBeU7dDCvDrSqRmq3VBeYY56H",
            Charity::OneTreePlanted => "Eq3eFm5ixRL73WDVw13AU6mzA9bkRHGyhwqBmRMJ6DZT",
            Charity::EvidenceAction => "HSpwMSrQKq8Zn3vJ6weNTuPtgNyEucTPpb8CtLXBZ6pQ",
            Charity::GirlsWhoCode => "GfhUjLFe6hewxqeV3SabB6jEARJw52gK8xuXecKCHA8U",
            Charity::OutrightActionInt => "4BMqPdMjtiCPGJ8G2ysKaU9zk55P7ANJNJ7T6XqzW6ns",
            Charity::TheLifeYouCanSave => "7LjZQ1UTgnsGUSnqBeiz3E4EofGA4e861wTBEixXFB6G"
        };

        let charity_index_map: EnumMap<Charity, usize> = enum_map!{
            Charity::UkraineERF => 0,
            Charity::WaterOrg => 1,
            Charity::OneTreePlanted => 2,
            Charity::EvidenceAction => 3,
            Charity::GirlsWhoCode => 4,
            Charity::OutrightActionInt => 5,
            Charity::TheLifeYouCanSave => 6
        };


        let account_info_iter = &mut accounts.iter();

        let bidder_account_info = next_account_info(account_info_iter)?;
        let bidder_token_account_info = next_account_info(account_info_iter)?;
        let bidder_data_account_info = next_account_info(account_info_iter)?;


        let dao_plays_account_info = next_account_info(account_info_iter)?;
        let charity_account_info = next_account_info(account_info_iter)?;

        let program_data_account_info = next_account_info(account_info_iter)?;
        let program_token_account_info = next_account_info(account_info_iter)?;


        let token_mint_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;


        if !bidder_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the second account should be the bidders associated token account
        let expected_bidder_token_key = get_associated_token_address(
            &bidder_account_info.key, 
            &token_mint_account_info.key
        );

        if bidder_token_account_info.key != &expected_bidder_token_key
        { 
            msg!("expected second account to be the player's associated token account {}", expected_bidder_token_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the third account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected third account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the seventh account is the charity SOL address, which we can check with the map
        let expected_charity_key = Pubkey::from_str(charity_key_map[bid_data.charity]).unwrap();

        if charity_account_info.key != &expected_charity_key
        {
            msg!("expected fifth account to be the chosen charities address {}", expected_charity_key);
            return Err(ProgramError::InvalidAccountData);
        }


        // the next account is the programs data account
        if program_data_account_info.key != &accounts::get_expected_data_account_key(program_id) {
            msg!("expected second account to be program data account {}", accounts::get_expected_data_account_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }

        let (expected_bidder_data_key, bidder_bump_seed) = Pubkey::find_program_address(&[&bidder_account_info.key.to_bytes()], &program_id);
        
        if bidder_data_account_info.key != &expected_bidder_data_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_bidder_data_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the fourth account should be the programs token address
        if program_token_account_info.key != &accounts::get_expected_program_token_key(program_id)
        { 
            msg!("expected fifth account to be the programs token account {}", accounts::get_expected_program_token_key(program_id));
            return Err(ProgramError::InvalidAccountData); 
        }


        // the fifth account is the mint address for the token
        if token_mint_account_info.key != &accounts::get_expected_token_mint_key()
        {
            msg!("expected fifth account to be the token mint address {}", accounts::get_expected_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the sixth  account is the token_program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected sixth account to be the token program {}", spl_token::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the seventh account is the associated_token_program
        if associated_token_account_info.key != &spl_associated_token_account::id() {
            msg!("expected twelfth account to be the associated token program {}", spl_associated_token_account::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eighth and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eighth account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // create the bidders token account if necessary
        utils::create_token_account(
            bidder_account_info,
            bidder_account_info,
            token_mint_account_info,
            bidder_token_account_info,
            token_program_account_info
        )?;
        
        
        // transfer the SOL to the required accounts
        let min_amount : u64 = 100000;
        if bid_data.amount_charity + bid_data.amount_dao < min_amount {
            msg!("Amount bid is less than the minimum of 0.0001 SOL");
            return Err(ProgramError::InvalidArgument);
        }

        // if we have made it this far in the transaction we can try transferring the SOL
        invoke(
            &system_instruction::transfer(bidder_account_info.key, charity_account_info.key, bid_data.amount_charity),
            &[bidder_account_info.clone(), charity_account_info.clone()],
        )?;

        invoke(
            &system_instruction::transfer(bidder_account_info.key, dao_plays_account_info.key, bid_data.amount_dao),
            &[bidder_account_info.clone(), dao_plays_account_info.clone()],
        )?;


        // update the data
        let charity_data_idx = get_state_index(StateEnum::CharityData);
        msg!("get charity data {} {} {} {}", charity_data_idx.0, charity_data_idx.1, charity_data_idx.1 - charity_data_idx.0, get_charity_size());
        let mut current_state = CharityData::try_from_slice(&program_data_account_info.data.borrow()[charity_data_idx.0..charity_data_idx.1])?;

        // calculate the current average to see if this individual has paid more
        let total_paid = bid_data.amount_charity + bid_data.amount_dao;

        let charity_index = charity_index_map[bid_data.charity];

        current_state.charity_totals[charity_index] += bid_data.amount_charity;
        current_state.donated_total += bid_data.amount_charity;
        current_state.paid_total += total_paid;
        current_state.n_donations += 1;

        msg!("Updating current state: {} {} {} {}", current_state.charity_totals[charity_index], current_state.donated_total, current_state.paid_total,  current_state.n_donations);

        current_state.serialize(&mut &mut program_data_account_info.data.borrow_mut()[charity_data_idx.0..charity_data_idx.1])?;
        

        // create the bidders data account if we need it
        utils::create_bidder_data_account(
            bidder_account_info,
            bidder_data_account_info,
            program_id,
            bidder_bump_seed
        )?;

        // get the current total so that we can increment it
        let bid_total_idx = get_state_index(StateEnum::TotalBidAmount);

        let mut current_bid_total = u64::try_from_slice(&program_data_account_info.data.borrow()[bid_total_idx.0..bid_total_idx.1])?;

        msg!("current bid total: {} {}", (current_bid_total as f64) / (LAMPORTS_PER_SOL as f64), (total_paid as f64) / (LAMPORTS_PER_SOL as f64));
        current_bid_total += total_paid;

        current_bid_total.serialize(&mut &mut program_data_account_info.data.borrow_mut()[bid_total_idx.0..bid_total_idx.1])?;


        // get the bid index from the bidders account
        let bid_status = BidderData::try_from_slice(&bidder_data_account_info.data.borrow()[..])?;
        msg!("{}", bid_status.index);

        let mut new_bid = total_paid;

        let bid_index = bid_status.index;

        // check the public key that is present in the data account at bid_index
        let key_idx = get_state_index(StateEnum::BidKeys{index: bid_index});
        let key = Pubkey::try_from_slice(&program_data_account_info.data.borrow()[key_idx.0..key_idx.1])?;

        msg!("compare keys {} {}", key, bidder_token_account_info.key);

        // if the keys match then we accumulate the bid
        // otherwise it must be a new bid
        if key == *bidder_token_account_info.key {

            msg!("Existing bid found, accumulating amount");
            // get the old bid
            let bid_idx = get_state_index(StateEnum::BidAmounts{index: bid_index});
            let bid =  u64::try_from_slice(&program_data_account_info.data.borrow()[bid_idx.0..bid_idx.1])?;
                            
            msg!("have old bid {}", bid);
            new_bid += bid;

            // and then update it
            new_bid.serialize(&mut &mut program_data_account_info.data.borrow_mut()[bid_idx.0..bid_idx.1])?; 
        }

        else {
        
            // if they were a new bidder add their bid to the ladder
            let bid_index_idx = get_state_index(StateEnum::BidIndex);
            let mut bid_index = usize::try_from_slice(&program_data_account_info.data.borrow()[bid_index_idx.0..bid_index_idx.1])?;  

            msg!("current bid position is {}", bid_index);

            let new_bid_idx = get_state_index(StateEnum::BidAmounts{index: bid_index});
            let new_key_idx = get_state_index(StateEnum::BidKeys{index: bid_index});

            //check if bidder's pubkey is already present
            let bidder_token_pubkey = *bidder_token_account_info.key;
        
            // serialise the new bid
            new_bid.serialize(&mut &mut program_data_account_info.data.borrow_mut()[new_bid_idx.0..new_bid_idx.1])?; 

            // serialise the new account
            bidder_token_pubkey.serialize(&mut &mut program_data_account_info.data.borrow_mut()[new_key_idx.0..new_key_idx.1])?;  

            // update their bid data
            let new_bidder_data = BidderData {index: bid_index};
            new_bidder_data.serialize(&mut &mut bidder_data_account_info.data.borrow_mut()[..])?;
            
            // update bid index
            bid_index = (bid_index + 1)%1024;
            msg!("update bid index: {}", bid_index);
            bid_index.serialize(&mut &mut program_data_account_info.data.borrow_mut()[bid_index_idx.0..bid_index_idx.1])?;  

            // update n_bidders
            let n_bidders_idx = get_state_index(StateEnum::NBidders);
            let mut n_bidders = u32::try_from_slice(&program_data_account_info.data.borrow()[n_bidders_idx.0..n_bidders_idx.1])?;
            n_bidders += 1;
            n_bidders.serialize(&mut &mut program_data_account_info.data.borrow_mut()[n_bidders_idx.0..n_bidders_idx.1])?; 
        }

        // Whenever anyone presses a button or bids for tokens we check whether it is a good time to select new token winners.
        // This uses the basic random generator, however no winners are selected at this point, we just update a bool in the program data.
        // A separate call to the program then decides the winner using Pyth oracles to seed the random number generators.
        utils::check_bid_state(program_data_account_info, program_token_account_info)?;

        Ok(())
    }

}