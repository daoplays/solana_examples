use std::str::FromStr;
use crate::state::{StateEnum, get_state_index, Charity, CharityData, BidderData, BidValues, MAX_WINNERS, TOKENS_WON, WinnersKeys, BID_BLOCK, N_BID_BLOCKS, BidTimes};
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

        if metadata.amount > 0 {
            utils::transfer_tokens(
                metadata.amount,
                token_source_account_info,
                program_token_account_info,
                funding_account_info,
                token_program_account_info,
                bump_seed
        
            )?;
        }

        // now just initialise the prev_selected_time field of the state to clock now
        

        let prev_time_idx = get_state_index(StateEnum::PrevSelectionTime);

        // check if the time is uninitialized and set it to the current time if so
        let prev_time_selected = i64::try_from_slice(&program_data_account_info.data.borrow()[prev_time_idx.0..prev_time_idx.1])?;

        if prev_time_selected == 0 {

            let clock = Clock::get()?;
            let current_time = clock.unix_timestamp;
            current_time.serialize(&mut &mut program_data_account_info.data.borrow_mut()[prev_time_idx.0..prev_time_idx.1])?;  
        }

        Ok(())


    }

    fn send_tokens(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) ->ProgramResult {

        msg!("in send_tokens");

        let account_info_iter = &mut accounts.iter().peekable();

        // first load and check all the non-winner accounts
        let funding_account_info = next_account_info(account_info_iter)?;
        let program_derived_account_info = next_account_info(account_info_iter)?;
        let program_token_account_info = next_account_info(account_info_iter)?;
        let program_data_account_info = next_account_info(account_info_iter)?;
        let token_program_account_info = next_account_info(account_info_iter)?;


        // the first account should be the funding account and should be a signer
        if !funding_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the second account is the program derived address which we can verify with find_program_address
        let (expected_pda_key, bump_seed) = accounts::get_expected_program_address_key(program_id);
         
        if program_derived_account_info.key != &expected_pda_key {
            msg!("expected second account to be PDA {}", expected_pda_key);
            return Err(ProgramError::InvalidAccountData);
        }

        // the third account is the program's token account
        if program_token_account_info.key != &accounts::get_expected_program_token_key(program_id) {
            msg!("expected third account to be the program's token account {}", accounts::get_expected_program_token_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }

        // the fifth account is the programs data account
        if program_data_account_info.key != &accounts::get_expected_data_account_key(program_id) {
            msg!("expected fifth account to be program data account {}", accounts::get_expected_data_account_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }

        // the sixth account is the token_program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected sixth account to be the token program {}", spl_token::id());
            return Err(ProgramError::InvalidAccountData);
        }

        
        // now check how many winners we expect and make sure the keys match the program data
        let n_winners_idx = get_state_index(StateEnum::NWinners);
        let n_winners = u8::try_from_slice(&program_data_account_info.data.borrow()[n_winners_idx.0..n_winners_idx.1])?;

        if n_winners == 0 {
            msg!("No winners selected, exiting send_tokens");
            return Ok(());
        }

        msg!("have {} winners to send tokens to", n_winners);

        // get the winner's account info
        let mut winners_account_info : Vec<&AccountInfo> = Vec::new();
        for _w_idx in 0..n_winners {

            if account_info_iter.peek().is_some() {
                winners_account_info.push(next_account_info(account_info_iter)?);
            }
            else {
                msg!("n_winners {} exceeds the number of accounts passed", n_winners);
                return Ok(());
            }
        }

        // check that was the last account
        if account_info_iter.peek().is_some() {
            msg!("n_winners {} is less than the number of accounts passed", n_winners);
            return Ok(());
        }

        let winners_key_idx = get_state_index(StateEnum::Winners { index: 0 });
        let expected_winners = WinnersKeys::try_from_slice(&program_data_account_info.data.borrow()[winners_key_idx.0..winners_key_idx.0 + 32 * MAX_WINNERS])?;

        // check the winners sent are what we expect
        // the front end may end up sending multiple requests to send tokens and we don't want the whole
        // instruction chain to fail just because the program state has moved on
        for w_idx in 0..(n_winners as usize) {
            msg!("winner {} : {}", w_idx, expected_winners.keys[w_idx].to_string());

            if expected_winners.keys[w_idx as usize] != *winners_account_info[w_idx].key {
                msg!("expected winner {} to have key {}", w_idx, winners_account_info[w_idx].key);
                return Ok(());
            }

            // also check none of the winners are the system program which would indicate we have arrived here too early
            if *winners_account_info[w_idx].key == solana_program::system_program::id() {
                msg!("winner {} has system program key {}", w_idx, winners_account_info[w_idx].key);
                return Ok(());
            }
        }

        // finally check that the remaining entries in the winners data vec are the system program id
        for w_idx in (n_winners as usize)..MAX_WINNERS {
            msg!("winner {} : {}", w_idx, expected_winners.keys[w_idx as usize].to_string());

            if expected_winners.keys[w_idx] != solana_program::system_program::id() {
                msg!("expected winner {} to have key {}", w_idx, solana_program::system_program::id());
                return Ok(());
            }
        }


        // now we can transfer the tokens

        for w_idx in 0..(n_winners as usize) {

            utils::transfer_tokens(
                TOKENS_WON,
                program_token_account_info,
                winners_account_info[w_idx],
                program_derived_account_info,
                token_program_account_info,
                bump_seed
        
            )?;
        }

        // finally just reset the n_winners value to zero so we can select new winners again
        // and reset all the winners keys to their default
        for current_winner in 0..MAX_WINNERS {

            let winner_idx = get_state_index(StateEnum::Winners{index: current_winner});
            solana_program::system_program::id().serialize(&mut &mut program_data_account_info.data.borrow_mut()[winner_idx.0..winner_idx.1])?; 

        }

        0u8.serialize(&mut &mut program_data_account_info.data.borrow_mut()[n_winners_idx.0..n_winners_idx.1])?;

        // as a sanity check  make sure the bidder data is still correct

        // calculate the total bid amount and number of bidders at this time
        let update = utils::get_bid_state(i64::MAX, program_data_account_info)?;
        let n_bidders = update.0;
        let total_bid = update.1;

        let n_bidders_idx = get_state_index(StateEnum::NBidders);
        let total_bid_idx = get_state_index(StateEnum::TotalBidAmount);

        let current_n_bidders =  u16::try_from_slice(&program_data_account_info.data.borrow()[n_bidders_idx.0..n_bidders_idx.1])?;
        let current_total_bid =  u64::try_from_slice(&program_data_account_info.data.borrow()[total_bid_idx.0..total_bid_idx.1])?;

        // check these agree

        if n_bidders != current_n_bidders || total_bid != current_total_bid {

            msg!("bid data is out of sync: {} {} {} {}", n_bidders, current_n_bidders, total_bid, current_total_bid);
            
            n_bidders.serialize(&mut &mut program_data_account_info.data.borrow_mut()[n_bidders_idx.0..n_bidders_idx.1])?;

            total_bid.serialize(&mut &mut program_data_account_info.data.borrow_mut()[total_bid_idx.0..total_bid_idx.1])?;
        }

    
        Ok(())
    }

    fn select_winners(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) ->ProgramResult {

   
        // we will use 3 streams, BTC,  ETH and SOL

        let account_info_iter = &mut accounts.iter();

        let funding_account_info = next_account_info(account_info_iter)?;


        // first accounts are the pyth oracles
        let btc_account_info = next_account_info(account_info_iter)?;
        let eth_account_info = next_account_info(account_info_iter)?;
        let sol_account_info = next_account_info(account_info_iter)?;

        let program_data_account_info = next_account_info(account_info_iter)?;
        let program_token_account_info = next_account_info(account_info_iter)?;


        // the first account should be the funding account and should be a signer
        if !funding_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
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

        // the last account should be the programs token address
        if program_token_account_info.key != &accounts::get_expected_program_token_key(program_id)
        { 
            msg!("expected sixth account to be the programs token account {}", accounts::get_expected_program_token_key(program_id));
            return Err(ProgramError::InvalidAccountData); 
        }

        // first check we should actually be here
        // if we have already chosen winners then we don't need to do anything

        let n_winners_idx = get_state_index(StateEnum::NWinners);
        let mut n_winners = u8::try_from_slice(&program_data_account_info.data.borrow()[n_winners_idx.0..n_winners_idx.1])?;

        if n_winners != 0 {
            msg!("winners have already been selected");
            return Ok(());
        }

        // update the prev_selected_time field of the state to clock now
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;
        let threshold_time = current_time - 2;

        // get the current total bid and n_bidders so we can update this later
        let n_bidders_idx = get_state_index(StateEnum::NBidders);
        let total_bid_idx = get_state_index(StateEnum::TotalBidAmount);

        let mut n_bidders =  u16::try_from_slice(&program_data_account_info.data.borrow()[n_bidders_idx.0..n_bidders_idx.1])?;
        let mut total_bid =  u64::try_from_slice(&program_data_account_info.data.borrow()[total_bid_idx.0..total_bid_idx.1])?;
        
        // for selecting winners we only include bids that were made up to a couple of seconds ago
        // and so want to find the total bid amount of just those
        let update = utils::get_bid_state(threshold_time, program_data_account_info)?;
        let mut valid_n_bidders = update.0;
        let mut valid_total_bid = update.1;

   
        // check to see if now is a good time to choose winners
        n_winners = utils::check_winners_state(
            valid_n_bidders, 
            program_data_account_info,
            program_token_account_info
        )?;
        
        msg!("check bids : {} {}, bid totals {} {} winners {}", valid_n_bidders, n_bidders, utils::to_sol(valid_total_bid), utils::to_sol(total_bid), n_winners);

        // if it is still zero though then we should just exit
        if n_winners == 0 || valid_n_bidders == 0 {
            msg!("No need to select winners, exiting {} {}", n_winners, valid_n_bidders);
            return Ok(());
        }

        // update n_winners
        n_winners.serialize(&mut &mut program_data_account_info.data.borrow_mut()[n_winners_idx.0..n_winners_idx.1])?;

        // generate the seed for selecting winners
        let mut pyth_random = randoms::generate_seed(
            btc_account_info,
            eth_account_info,
            sol_account_info
        );

        let mut ran_vec : Vec<f64> = Vec::new();
        for _winner in 0..n_winners {
            pyth_random = randoms::shift_seed(pyth_random);
            let random_f64 = randoms::generate_random(pyth_random);

            ran_vec.push(random_f64);
            //msg!("random {} : {}", winner, random_f64);

        }

        ran_vec.sort_by(|a, b| a.partial_cmp(b).unwrap());


        // get the starting state
        let mut cumulative_total : u64 = 0;
        let mut winners_found : [bool; MAX_WINNERS] = [false; MAX_WINNERS];

        for idx in 0..N_BID_BLOCKS {

            let bid_idx = get_state_index(StateEnum::BidAmounts {index: idx*BID_BLOCK});
            let mut bids = BidValues::try_from_slice(&program_data_account_info.data.borrow()[bid_idx.0..bid_idx.0 + BID_BLOCK*8])?; 

            let time_idx = get_state_index(StateEnum::BidTimes {index: idx*BID_BLOCK});
            let times = BidTimes::try_from_slice(&program_data_account_info.data.borrow()[time_idx.0..time_idx.0 + BID_BLOCK*8])?; 
    
            for current_winner in 0..n_winners {

                if winners_found[current_winner as usize] {
                    continue;
                }

                // update the threshold
                let random_f64 = ran_vec[current_winner as usize];
                let threshold = ((valid_total_bid as f64) * random_f64) as u64;

                //msg!("check for winner {} block {} total {} threshold {} current total {}", current_winner, idx,utils::to_sol(valid_total_bid), utils::to_sol(threshold),  utils::to_sol(cumulative_total));

                let mut sub_total : u64 = cumulative_total;
                for bid_index in 0..BID_BLOCK {

                    // check if this is within the time threshold
                    if times.bid_times[bid_index] >= threshold_time {
                        continue;
                    }
                    
                    let current_bid =  bids.bid_amounts[bid_index];
                    sub_total += current_bid;
        
                    if sub_total > threshold {

                        winners_found[current_winner as usize] = true;
        
                        let winner_index = idx * BID_BLOCK + bid_index;
        
                        msg!("Have winner {}: idx {}, random = {},  {} > {}, bid {}", current_winner, winner_index, random_f64, (sub_total as f64) / (LAMPORTS_PER_SOL as f64), (threshold as f64) / (LAMPORTS_PER_SOL as f64), (current_bid as f64) / (LAMPORTS_PER_SOL as f64));

                        // get the winners key from the program data account
                        let key_idx = get_state_index(StateEnum::BidKeys{index: winner_index});
                        let winners_key = Pubkey::try_from_slice(&program_data_account_info.data.borrow()[key_idx.0..key_idx.1])?; 
        
                        // and insert it into the winners array
                        let winner_idx = get_state_index(StateEnum::Winners{index: current_winner as usize});
                        winners_key.serialize(&mut &mut program_data_account_info.data.borrow_mut()[winner_idx.0..winner_idx.1])?; 

    
                        // now clear the winners data in the program data account
                        // start by zero'ing their bid
                        let win_bid_idx = get_state_index(StateEnum::BidAmounts {index: winner_index});
                        0u64.serialize(&mut &mut program_data_account_info.data.borrow_mut()[win_bid_idx.0..win_bid_idx.1])?;  

                        // then the bid time
                        let win_time_idx = get_state_index(StateEnum::BidTimes{index: winner_index});
                        0i64.serialize(&mut &mut program_data_account_info.data.borrow_mut()[win_time_idx.0..win_time_idx.1])?;  

                        // and then clear their key
                        solana_program::system_program::id().serialize(&mut &mut program_data_account_info.data.borrow_mut()[key_idx.0..key_idx.1])?;
     
                        // as a sanity check make sure current bid is less than total_bid
                        if current_bid > valid_total_bid {
                            msg!("Current bid is greater than total bid, this shouldn't happen {} > {}", current_bid, valid_total_bid);
                            return Ok(());
                        }

                        // finally decrement the number of bidders, and the total bid amount
                        valid_n_bidders -= 1;
                        valid_total_bid -= current_bid;

                        n_bidders -= 1;
                        total_bid -= current_bid;

                        bids.bid_amounts[bid_index] = 0;

                        break;

                    }
                }

                // if this winner wasn't found in this block, move onto the next block
                if winners_found[current_winner as usize] == false {

                    cumulative_total = sub_total;
                   
                    break;
                }
            }
        }

        // update number of bidders
        let n_bidders_idx = get_state_index(StateEnum::NBidders);
        n_bidders.serialize(&mut &mut program_data_account_info.data.borrow_mut()[n_bidders_idx.0..n_bidders_idx.1])?;

        // update total_bid_amount
        let total_bid_idx = get_state_index(StateEnum::TotalBidAmount);
        total_bid.serialize(&mut &mut program_data_account_info.data.borrow_mut()[total_bid_idx.0..total_bid_idx.1])?;


        let prev_time_idx = get_state_index(StateEnum::PrevSelectionTime);
        current_time.serialize(&mut &mut program_data_account_info.data.borrow_mut()[prev_time_idx.0..prev_time_idx.1])?;  

     
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


        // update the charity stats data
        let charity_data_idx = get_state_index(StateEnum::CharityData);
        //msg!("get charity data {} {} {} {}", charity_data_idx.0, charity_data_idx.1, charity_data_idx.1 - charity_data_idx.0, get_charity_size());
        let mut current_state = CharityData::try_from_slice(&program_data_account_info.data.borrow()[charity_data_idx.0..charity_data_idx.1])?;

        // calculate the current average to see if this individual has paid more
        let total_paid = bid_data.amount_charity + bid_data.amount_dao;

        let charity_index = charity_index_map[bid_data.charity];

        current_state.charity_totals[charity_index] += bid_data.amount_charity;
        current_state.donated_total += bid_data.amount_charity;
        current_state.paid_total += total_paid;
        current_state.n_donations += 1;

        current_state.serialize(&mut &mut program_data_account_info.data.borrow_mut()[charity_data_idx.0..charity_data_idx.1])?;
        

        // create the bidders data account if we need it
        utils::create_bidder_data_account(
            bidder_account_info,
            bidder_data_account_info,
            program_id,
            bidder_bump_seed
        )?;

        // we will need to update n_bidders and total_bid so get them now
        let n_bidders_idx = get_state_index(StateEnum::NBidders);
        let total_bid_idx = get_state_index(StateEnum::TotalBidAmount);

        let mut n_bidders =  u16::try_from_slice(&program_data_account_info.data.borrow()[n_bidders_idx.0..n_bidders_idx.1])?;
        let mut total_bid =  u64::try_from_slice(&program_data_account_info.data.borrow()[total_bid_idx.0..total_bid_idx.1])?;


        let mut new_bid = total_paid;

        // update total_bid with new_bid
        total_bid += new_bid;

        
        // get the current time as a point of comparison for finding the oldest bid, and for the bids time
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // get the bid index from the bidders account
        let bidder_data = BidderData::try_from_slice(&bidder_data_account_info.data.borrow()[..])?;


        // when adding the bid to the program state we have three possibilities:
        // i) there is already a bid and we just accumulate
        // ii) there is no bid but there is an empty spot
        // iii) there is no bid and no empty spot, so we replace the oldest bid

        // start by checking if a bid exists
        let mut bidders_index = bidder_data.index as usize;

        // check the public key that is present in the data account at bid_index
        let key_idx = get_state_index(StateEnum::BidKeys{index: bidders_index});
        let key = Pubkey::try_from_slice(&program_data_account_info.data.borrow()[key_idx.0..key_idx.1])?;

        msg!("compare keys {} {} as position  {}", key, bidder_token_account_info.key, bidder_data.index);
        
        // if the keys match then we accumulate the bid
        // otherwise it must be a new bid
        if key == *bidder_token_account_info.key {

            msg!("Existing bid found, accumulating amount");
            // get the old bid
            let old_bid_idx = get_state_index(StateEnum::BidAmounts{index: bidders_index});
            let old_bid =  u64::try_from_slice(&program_data_account_info.data.borrow()[old_bid_idx.0..old_bid_idx.1])?;
                            
            msg!("have old bid {} + {} -> {}", old_bid, new_bid, new_bid + old_bid);
            new_bid += old_bid;
 
        }

        else {

            // if they were a new bidder add their bid to the ladder, first just try and find the first open spot
            msg!("Have new bidder");
            let mut found_space = false;
             
            // if there isn't a space we will want to replace the oldest bid
            // so we find  that in the same loop
            let mut oldest_bid_index : usize = 0;
            let mut oldest_time = i64::MAX;
            for i in 0..N_BID_BLOCKS {


                let time_idx = get_state_index(StateEnum::BidTimes {index: i * BID_BLOCK});
                let times = BidTimes::try_from_slice(&program_data_account_info.data.borrow()[time_idx.0..time_idx.0 + BID_BLOCK * 8])?; 
        
                for j in 0..BID_BLOCK {

                    let total_index = i * BID_BLOCK + j;

                    // if the bid time is zero that indicates we have found an empty slot, so break out of the loop
                    if times.bid_times[j] == 0 {
                        bidders_index = total_index;
                        found_space = true;
                        break
                    }

                    // otherwise check if this is older than the oldest known bid so far
                    if times.bid_times[j] < oldest_time {
                        oldest_bid_index = total_index;
                        oldest_time = times.bid_times[j];
                    }
                }

                if found_space {
                    break;
                }
            }

            
  
            // if there was no open spot we overwrite the oldest bid
            if !found_space {

                bidders_index = oldest_bid_index;
                msg!("using oldest bid position in {}", bidders_index);

                // if we are overwriting we need to subtract bid_amount and reduce n_bidders by one
                let existing_bid_idx = get_state_index(StateEnum::BidAmounts{index: bidders_index});
                let existing_bid = u64::try_from_slice(&program_data_account_info.data.borrow()[existing_bid_idx.0..existing_bid_idx.1])?;
                total_bid -= existing_bid;
                n_bidders -=1;

            }

            // for a new bid we need to add the public key
            let bidder_token_pubkey = *bidder_token_account_info.key;
            let new_key_idx = get_state_index(StateEnum::BidKeys{index: bidders_index});
        
            // serialise the new account
            bidder_token_pubkey.serialize(&mut &mut program_data_account_info.data.borrow_mut()[new_key_idx.0..new_key_idx.1])?;  

            // update their bid data
            let new_bidder_data = BidderData {index: bidders_index as u16};
            new_bidder_data.serialize(&mut &mut bidder_data_account_info.data.borrow_mut()[..])?;

            // update n_bidders
            n_bidders += 1;
     
        }

        msg!("update bid details for position {}", bidders_index);

        // insert the new bid and time into the program data
        let new_bid_idx = get_state_index(StateEnum::BidAmounts{index: bidders_index});
        let new_time_idx = get_state_index(StateEnum::BidTimes{index: bidders_index});

        current_time.serialize(&mut &mut program_data_account_info.data.borrow_mut()[new_time_idx.0..new_time_idx.1])?;  
        new_bid.serialize(&mut &mut program_data_account_info.data.borrow_mut()[new_bid_idx.0..new_bid_idx.1])?; 

        // update total bid
        total_bid.serialize(&mut &mut program_data_account_info.data.borrow_mut()[total_bid_idx.0..total_bid_idx.1])?; 

        //  update n_bidders
        n_bidders.serialize(&mut &mut program_data_account_info.data.borrow_mut()[n_bidders_idx.0..n_bidders_idx.1])?; 


        Ok(())
    }

}