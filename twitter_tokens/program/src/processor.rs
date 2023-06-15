use std::str::FromStr;
use crate::instruction::ErrorMeta;
use crate::state::{UserData,  IDMap};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::native_token::LAMPORTS_PER_SOL;
use crate::accounts;
use crate::utils;
use crate::state;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    msg,
    pubkey::Pubkey,
    program::{invoke},
    system_instruction,
    clock::Clock, sysvar::Sysvar, rent
};
use spl_associated_token_account::get_associated_token_address;

use crate::{instruction::{TwitterInstruction, RegisterMeta, UserMeta, TokenMeta, HashTagMeta, HashTagRewardMeta}};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        let instruction = TwitterInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            TwitterInstruction::InitProgram {metadata} => {
                msg!("Instruction: Init Program");
                Self::init_program(accounts, program_id, metadata)
            },
            TwitterInstruction::Register {metadata} => {
                msg!("Instruction: Register");
                Self::register_user(accounts, program_id, metadata)
            },
            TwitterInstruction::CreateUserAccount => {
                msg!("Instruction: Create User Account");
                Self::create_user_account(accounts, program_id)
            },
            TwitterInstruction::NewFollower {metadata}  => {
                msg!("Instruction: New Follower");
                Self::check_new_follower(accounts, program_id, metadata)
            },
            TwitterInstruction::SetError {metadata}  => {
                msg!("Instruction: Set Error");
                Self::set_error(accounts, program_id, metadata)
            },
            TwitterInstruction::CheckFollower => {
                msg!("Instruction: Check Follower");
                Self::check_follower(accounts, program_id)
            },
            TwitterInstruction::CheckHashTag {metadata}  => {
                msg!("Instruction: Check Hashtag");
                Self::check_hashtag(accounts, program_id, metadata)
            },
            TwitterInstruction::SendTokens {metadata}  => {
                msg!("Instruction: Send Tokens");
                Self::send_hashtag_reward(accounts, program_id, metadata)
            },
            TwitterInstruction::CheckRetweet {metadata}  => {
                msg!("Instruction: Check Retweet");
                Self::check_retweet(accounts, program_id, metadata)
            },
            TwitterInstruction::SetUserID {metadata}  => {
                msg!("Instruction: Set User ID");
                Self::set_user_id(accounts, program_id, metadata)
            }
        }
    } 
 

    fn init_program(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : TokenMeta
    ) ->ProgramResult 
    {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed eight accounts, get them all first and then check their value is as expected
        let funding_account_info = next_account_info(account_info_iter)?;
        let program_derived_account_info = next_account_info(account_info_iter)?;

        let supporters_token_source_account_info = next_account_info(account_info_iter)?;
        let program_supporters_token_account_info = next_account_info(account_info_iter)?;
        let supporters_token_mint_account_info = next_account_info(account_info_iter)?;

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
  

        // the sixth account is the source of the supporter tokens
        if supporters_token_source_account_info.key != &accounts::get_expected_daoplays_supporters_token_key() {
            msg!("expected sixth account to be the funder's supporter token account {}", accounts::get_expected_daoplays_supporters_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the seventh account is the program's supporters token account
        if program_supporters_token_account_info.key != &accounts::get_expected_program_supporter_token_key(program_id) {
            msg!("expected seventh account to be the program's supporters token account {}", accounts::get_expected_program_supporter_token_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }

        // the eighth account is the mint address for the supporter token
        if supporters_token_mint_account_info.key != &accounts::get_expected_supporter_token_mint_key() {
            msg!("expected eighth account to be the supporter token's mint account {}", accounts::get_expected_supporter_token_mint_key());
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
            supporters_token_mint_account_info,
            program_supporters_token_account_info,
            token_program_account_info
        )?;
        
        utils::transfer_tokens(
            metadata.amount,
            supporters_token_source_account_info,
            program_supporters_token_account_info,
            funding_account_info,
            token_program_account_info,
            bump_seed
    
        )?;


        Ok(())

    }

    fn register_user(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        _metadata : RegisterMeta
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let user_account_info = next_account_info(account_info_iter)?;
        let user_supporters_token_account_info = next_account_info(account_info_iter)?;
        let user_id_map_account_info = next_account_info(account_info_iter)?;

        let dao_plays_account_info = next_account_info(account_info_iter)?;

        let supporters_token_mint_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !user_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }



        // the third account should be the joiners supporter associated token account
        let expected_user_supporters_token_key = get_associated_token_address(
            &user_account_info.key, 
            &supporters_token_mint_account_info.key
        );

        if user_supporters_token_account_info.key != &expected_user_supporters_token_key
        { 
            msg!("expected third account to be the joiner's supporter associated token account {}", expected_user_supporters_token_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        let (expected_user_data_key, user_bump_seed) = Pubkey::find_program_address(&[&user_account_info.key.to_bytes()], &program_id);
        
        if user_id_map_account_info.key != &expected_user_data_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_user_data_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the third account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected third account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the tenth account is the mint address for the supporters token
        if supporters_token_mint_account_info.key != &accounts::get_expected_supporter_token_mint_key()
        {
            msg!("expected tenth account to be the token mint address {}", accounts::get_expected_supporter_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eleventh account is the token_program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected eleventh account to be the token program {}", spl_token::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the twelfth account is the associated_token_program
        if associated_token_account_info.key != &spl_associated_token_account::id() {
            msg!("expected twelfth account to be the associated token program {}", spl_associated_token_account::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eleventh and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eleventh account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }


        // create the supporters token account if we need it
        utils::create_token_account(
            user_account_info,
            user_account_info,
            supporters_token_mint_account_info,
            user_supporters_token_account_info,
            token_program_account_info
        )?;

        // create the users data account if we need it
        utils::create_user_data_account(
            user_account_info,
            user_id_map_account_info,
            program_id,
            user_bump_seed,
            &user_account_info.key.to_bytes(),
            state::get_id_map_size()
        )?;

        msg!("account size: {}", state::get_id_map_size());

        let mut current_state = IDMap::try_from_slice(&user_id_map_account_info.data.borrow()[..])?;
        let twitter_id = current_state.twitter_id;
        msg!("twitter id: {}", twitter_id);

        current_state.error_code = 100;
        current_state.serialize(&mut &mut user_id_map_account_info.data.borrow_mut()[..])?;


        // transfer the lamports for us to init the user id map
        let transaction_cost_lamports : u64 = 5000;

        msg!("transferring {} SOL to init id map", utils::to_sol(transaction_cost_lamports));


        invoke(
            &system_instruction::transfer(user_account_info.key, dao_plays_account_info.key, transaction_cost_lamports),
            &[user_account_info.clone(), dao_plays_account_info.clone()],
        )?;

        Ok(())


    }

    fn set_user_id(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : UserMeta
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let dao_plays_account_info = next_account_info(account_info_iter)?;
        let user_account_info = next_account_info(account_info_iter)?;
        let user_id_map_account_info = next_account_info(account_info_iter)?;


        // the first account should be the funding account and should be a signer
        if !dao_plays_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the third account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected third account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        let (expected_user_id_map_key, _user_id_map_bump_seed) = Pubkey::find_program_address(&[&user_account_info.key.to_bytes()], &program_id);
        
        if user_id_map_account_info.key != &expected_user_id_map_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_user_id_map_key);
            return Err(ProgramError::InvalidAccountData); 
        }


        // update the id map
        let mut id_map = IDMap::try_from_slice(&user_id_map_account_info.data.borrow()[..])?;
        id_map.twitter_id = metadata.user_id;
        id_map.serialize(&mut &mut user_id_map_account_info.data.borrow_mut()[..])?;

        Ok(())


    }

    fn create_user_account(
        accounts: &[AccountInfo],
        program_id: &Pubkey
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let user_account_info = next_account_info(account_info_iter)?;
        let user_data_account_info = next_account_info(account_info_iter)?;
        let user_id_map_account_info = next_account_info(account_info_iter)?;

        let system_program_account_info = next_account_info(account_info_iter)?;

        let (expected_user_id_map_key, _user_id_map_bump_seed) = Pubkey::find_program_address(&[&user_account_info.key.to_bytes()], &program_id);
        
        if user_id_map_account_info.key != &expected_user_id_map_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_user_id_map_key);
            return Err(ProgramError::InvalidAccountData); 
        }


        // check that the user id account exists
        if **user_id_map_account_info.try_borrow_lamports()? == 0 {
            msg!("user's id map doesn't exist");
            return Err(ProgramError::InvalidAccountData);
        }

        let mut id_map = IDMap::try_from_slice(&user_id_map_account_info.data.borrow()[..])?;
        let twitter_id = id_map.twitter_id;

        // check that the twitter id has been set
        if twitter_id == 0 {
            msg!("user's id map not yet initialized");
            return Err(ProgramError::InvalidAccountData);
        }

        let (expected_user_data_key, user_bump_seed) = Pubkey::find_program_address(&[&twitter_id.to_le_bytes()], &program_id);
        
        if user_data_account_info.key != &expected_user_data_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_user_data_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the eleventh and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eleventh account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // create the users data account if we need it
        utils::create_user_data_account(
            user_account_info,
            user_data_account_info,
            program_id,
            user_bump_seed,
            &twitter_id.to_le_bytes(),
            state::get_user_data_size()
        )?;

        // init the data
        let mut current_state = UserData::try_from_slice(&user_data_account_info.data.borrow()[..])?;
        current_state.account_key = *user_account_info.key;
        current_state.serialize(&mut &mut user_data_account_info.data.borrow_mut()[..])?;

        id_map.error_code = 0;
        id_map.serialize(&mut &mut user_id_map_account_info.data.borrow_mut()[..])?;


        Ok(())


    }

    fn check_new_follower(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : UserMeta
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let dao_plays_account_info = next_account_info(account_info_iter)?;

        let user_account_info = next_account_info(account_info_iter)?;
        let user_data_account_info = next_account_info(account_info_iter)?;
        let user_supporter_token_account_info = next_account_info(account_info_iter)?;

        let program_derived_account_info = next_account_info(account_info_iter)?;
        let program_supporter_token_account_info = next_account_info(account_info_iter)?;

        let supporters_token_mint_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !dao_plays_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the third account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected third account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        let (expected_user_data_key, _user_bump_seed) = Pubkey::find_program_address(&[&metadata.user_id.to_le_bytes()], &program_id);
        
        if user_data_account_info.key != &expected_user_data_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_user_data_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the third account should be the joiners supporter associated token account
        let expected_user_supporters_token_key = get_associated_token_address(
            &user_account_info.key, 
            &supporters_token_mint_account_info.key
        );

        if user_supporter_token_account_info.key != &expected_user_supporters_token_key
        { 
            msg!("expected third account to be the joiner's supporter associated token account {}", expected_user_supporters_token_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the second account is the program derived address which we can verify with find_program_address
        let (expected_pda_key, bump_seed) = accounts::get_expected_program_address_key(program_id);
    
        if program_derived_account_info.key != &expected_pda_key {
            msg!("expected second account to be PDA {}", expected_pda_key);
            return Err(ProgramError::InvalidAccountData);
        }

        // the third account is the program's token account
        if program_supporter_token_account_info.key != &accounts::get_expected_program_supporter_token_key(program_id) {
            msg!("expected third account to be the program's token account {}", accounts::get_expected_program_supporter_token_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }

        // the tenth account is the mint address for the supporters token
        if supporters_token_mint_account_info.key != &accounts::get_expected_supporter_token_mint_key()
        {
            msg!("expected tenth account to be the token mint address {}", accounts::get_expected_supporter_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eleventh account is the token_program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected eleventh account to be the token program {}", spl_token::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the twelfth account is the associated_token_program
        if associated_token_account_info.key != &spl_associated_token_account::id() {
            msg!("expected twelfth account to be the associated token program {}", spl_associated_token_account::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eleventh and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eleventh account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }


        // check the data
        let mut current_state = UserData::try_from_slice(&user_data_account_info.data.borrow()[..])?;

        if current_state.account_key  != *user_account_info.key  {
            msg!("saved key doesn't match user account");
            return Ok(());
        }
        
        if current_state.follow {
            msg!("user is already following");
            return Ok(());
        }

        current_state.follow = true;

        current_state.serialize(&mut &mut user_data_account_info.data.borrow_mut()[..])?;

        utils::transfer_tokens(10,
            program_supporter_token_account_info,
            user_supporter_token_account_info,
            program_derived_account_info,
            token_program_account_info,
            bump_seed)?;

        Ok(())


    }

    fn set_error(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : ErrorMeta
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let dao_plays_account_info = next_account_info(account_info_iter)?;

        let user_account_info = next_account_info(account_info_iter)?;
        let user_id_map_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !dao_plays_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the third account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected third account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        let (expected_user_id_map_key, _user_id_map_bump_seed) = Pubkey::find_program_address(&[&user_account_info.key.to_bytes()], &program_id);
        
        if user_id_map_account_info.key != &expected_user_id_map_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_user_id_map_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // set the error code
        let mut current_state = IDMap::try_from_slice(&user_id_map_account_info.data.borrow()[..])?;
        current_state.error_code = metadata.error_code;
        current_state.serialize(&mut &mut user_id_map_account_info.data.borrow_mut()[..])?;

        Ok(())


    }

    fn check_follower(
        accounts: &[AccountInfo],
        _program_id: &Pubkey
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let user_account_info = next_account_info(account_info_iter)?;
        let dao_plays_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !user_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the third account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected third account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eleventh and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eleventh account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // transfer the lamports for us to send the tokens
        let transaction_cost_lamports : u64 = 5000;

        msg!("transferring {} SOL to send tokens", utils::to_sol(transaction_cost_lamports));


        invoke(
            &system_instruction::transfer(user_account_info.key, dao_plays_account_info.key, transaction_cost_lamports),
            &[user_account_info.clone(), dao_plays_account_info.clone()],
        )?;

        Ok(())


    }

    fn check_hashtag(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : HashTagMeta
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let user_account_info = next_account_info(account_info_iter)?;
        let user_id_map_account_info = next_account_info(account_info_iter)?;
        let user_hashtag_account_info = next_account_info(account_info_iter)?;

        let dao_plays_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !user_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (expected_user_id_map_key, _user_id_map_bump_seed) = Pubkey::find_program_address(&[&user_account_info.key.to_bytes()], &program_id);
        
        if user_id_map_account_info.key != &expected_user_id_map_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_user_id_map_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        let mut current_state = IDMap::try_from_slice(&user_id_map_account_info.data.borrow()[..])?;
        let user_id = current_state.twitter_id;

        let (expected_user_hashtag_key, user_hashtag_bump_seed) = Pubkey::find_program_address(&[metadata.hashtag.as_bytes(), &metadata.tweet_id.to_le_bytes(), &user_id.to_le_bytes()], &program_id);
        
        if user_hashtag_account_info.key != &expected_user_hashtag_key
        { 
            msg!("expected fifth account to be the hashtag data account {}", expected_user_hashtag_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the third account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected third account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eleventh and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eleventh account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // create the data account for this hashtag use
        utils::create_hashtag_data_account(
            user_account_info,
            user_hashtag_account_info,
            program_id,
            user_hashtag_bump_seed,
            metadata.hashtag.as_bytes(),
            &metadata.tweet_id.to_le_bytes(),
            &user_id.to_le_bytes(),
            state::get_mark_size()
        )?;

        // transfer the lamports for us to send the tokens
        let transaction_cost_lamports : u64 = 5000;

        msg!("transferring {} SOL to send tokens", utils::to_sol(transaction_cost_lamports));


        invoke(
            &system_instruction::transfer(user_account_info.key, dao_plays_account_info.key, transaction_cost_lamports),
            &[user_account_info.clone(), dao_plays_account_info.clone()],
        )?;

        current_state.error_code = 0;
        current_state.serialize(&mut &mut user_id_map_account_info.data.borrow_mut()[..])?;

        Ok(())


    }

    fn check_retweet(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : HashTagMeta
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let user_account_info = next_account_info(account_info_iter)?;
        let user_id_map_account_info = next_account_info(account_info_iter)?;
        let user_hashtag_account_info = next_account_info(account_info_iter)?;

        let dao_plays_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !user_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (expected_user_id_map_key, _user_id_map_bump_seed) = Pubkey::find_program_address(&[&user_account_info.key.to_bytes()], &program_id);
        
        if user_id_map_account_info.key != &expected_user_id_map_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_user_id_map_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        let mut current_state = IDMap::try_from_slice(&user_id_map_account_info.data.borrow()[..])?;
        let user_id = current_state.twitter_id;

        let (expected_user_hashtag_key, user_hashtag_bump_seed) = Pubkey::find_program_address(&["retweet".as_bytes(), &metadata.tweet_id.to_le_bytes(), &user_id.to_le_bytes()], &program_id);
        
        if user_hashtag_account_info.key != &expected_user_hashtag_key
        { 
            msg!("expected fifth account to be the hashtag data account {}", expected_user_hashtag_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the third account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected third account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eleventh and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eleventh account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // create the data account for this hashtag use
        utils::create_hashtag_data_account(
            user_account_info,
            user_hashtag_account_info,
            program_id,
            user_hashtag_bump_seed,
            "retweet".as_bytes(),
            &metadata.tweet_id.to_le_bytes(),
            &user_id.to_le_bytes(),
            state::get_mark_size()
        )?;

        // transfer the lamports for us to send the tokens
        let transaction_cost_lamports : u64 = 5000;

        msg!("transferring {} SOL to send tokens", utils::to_sol(transaction_cost_lamports));


        invoke(
            &system_instruction::transfer(user_account_info.key, dao_plays_account_info.key, transaction_cost_lamports),
            &[user_account_info.clone(), dao_plays_account_info.clone()],
        )?;

        current_state.error_code = 0;
        current_state.serialize(&mut &mut user_id_map_account_info.data.borrow_mut()[..])?;

        Ok(())


    }


    fn send_hashtag_reward(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        metadata : HashTagRewardMeta
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let dao_plays_account_info = next_account_info(account_info_iter)?;

        let user_account_info = next_account_info(account_info_iter)?;
        let user_id_map_account_info = next_account_info(account_info_iter)?;
        let user_data_account_info = next_account_info(account_info_iter)?;
        let user_hashtag_account_info = next_account_info(account_info_iter)?;
        let user_supporter_token_account_info = next_account_info(account_info_iter)?;

        let program_derived_account_info = next_account_info(account_info_iter)?;
        let program_supporter_token_account_info = next_account_info(account_info_iter)?;

        let supporters_token_mint_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        // the first account should be the funding account and should be a signer
        if !dao_plays_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the third account is the daoplays SOL address
        if dao_plays_account_info.key != &accounts::get_expected_daoplays_key()
        {
            msg!("expected third account to be the daoplays address {}", accounts::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        let (expected_user_id_map_key, _user_id_map_bump_seed) = Pubkey::find_program_address(&[&user_account_info.key.to_bytes()], &program_id);
        
        if user_id_map_account_info.key != &expected_user_id_map_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_user_id_map_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        let current_state = IDMap::try_from_slice(&user_id_map_account_info.data.borrow()[..])?;
        let user_id = current_state.twitter_id;

        let (expected_user_data_key, _user_bump_seed) = Pubkey::find_program_address(&[&user_id.to_le_bytes()], &program_id);
        
        if user_data_account_info.key != &expected_user_data_key
        { 
            msg!("expected fifth account to be the bidders data account {}", expected_user_data_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        let (expected_user_hashtag_key, _user_hashtag_bump_seed) = Pubkey::find_program_address(&[metadata.hashtag.as_bytes(), &metadata.tweet_id.to_le_bytes(), &user_id.to_le_bytes()], &program_id);
        
        if user_hashtag_account_info.key != &expected_user_hashtag_key
        { 
            msg!("expected fifth account to be the hashtag data account {}", expected_user_hashtag_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the third account should be the joiners supporter associated token account
        let expected_user_supporters_token_key = get_associated_token_address(
            &user_account_info.key, 
            &supporters_token_mint_account_info.key
        );

        if user_supporter_token_account_info.key != &expected_user_supporters_token_key
        { 
            msg!("expected third account to be the joiner's supporter associated token account {}", expected_user_supporters_token_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the second account is the program derived address which we can verify with find_program_address
        let (expected_pda_key, bump_seed) = accounts::get_expected_program_address_key(program_id);
    
        if program_derived_account_info.key != &expected_pda_key {
            msg!("expected second account to be PDA {}", expected_pda_key);
            return Err(ProgramError::InvalidAccountData);
        }

        // the third account is the program's token account
        if program_supporter_token_account_info.key != &accounts::get_expected_program_supporter_token_key(program_id) {
            msg!("expected third account to be the program's token account {}", accounts::get_expected_program_supporter_token_key(program_id));
            return Err(ProgramError::InvalidAccountData);
        }

        // the tenth account is the mint address for the supporters token
        if supporters_token_mint_account_info.key != &accounts::get_expected_supporter_token_mint_key()
        {
            msg!("expected tenth account to be the token mint address {}", accounts::get_expected_supporter_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eleventh account is the token_program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected eleventh account to be the token program {}", spl_token::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the twelfth account is the associated_token_program
        if associated_token_account_info.key != &spl_associated_token_account::id() {
            msg!("expected twelfth account to be the associated token program {}", spl_associated_token_account::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eleventh and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eleventh account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // check the user data
        let mut user_data = UserData::try_from_slice(&user_data_account_info.data.borrow()[..])?;

        if user_data.account_key  != *user_account_info.key  {
            msg!("saved key doesn't match user account");
            return Ok(());
        }

        // check when we last did a reward
        let last_utc_day_rewarded =  user_data.last_time / 86400;

        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;
        let current_day = current_time / 86400;

        msg!("compare days {} {}", last_utc_day_rewarded, current_day);

        if metadata.hashtag != "retweet" && current_day == last_utc_day_rewarded {
            msg!("already received a hashtag reward for day {}", current_day);
        }

        // check the reward data
        let mut reward_state = state::RewardMark::try_from_slice(&user_hashtag_account_info.data.borrow()[..])?;

       
        if reward_state.mark {
            msg!("reward has already been sent for this post");
            return Ok(());
        }

        // update the time
        
        user_data.last_time = current_time;

        user_data.serialize(&mut &mut user_data_account_info.data.borrow_mut()[..])?;

        msg!("current time is {}", current_time);

        reward_state.mark = true;

        reward_state.serialize(&mut &mut user_hashtag_account_info.data.borrow_mut()[..])?;

        utils::transfer_tokens(metadata.amount,
            program_supporter_token_account_info,
            user_supporter_token_account_info,
            program_derived_account_info,
            token_program_account_info,
            bump_seed)?;

        Ok(())


    }



}