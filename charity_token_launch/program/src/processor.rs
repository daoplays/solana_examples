use borsh::{BorshDeserialize, BorshSerialize};
use std::str::FromStr;
use crate::state::{JoinMeta, InitMeta, Charity, TokenLaunchData, get_state_size};
use enum_map::{enum_map, EnumMap};


use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,msg,
    program_error::ProgramError,
    program::invoke_signed,
    program::invoke,
    sysvar::rent,
    system_instruction, program_pack::Pack
};

use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};


use crate::{instruction::TokenLaunchInstruction};

pub struct Processor;
impl Processor {
    
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        let instruction = TokenLaunchInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            TokenLaunchInstruction::InitTokenLaunch {metadata} => {

                Self::init_token_launch(program_id, accounts, metadata)
            },
            TokenLaunchInstruction::JoinTokenLaunch {metadata} => {

                Self::join_token_launch(program_id, accounts, metadata)
            },
            TokenLaunchInstruction::EndTokenLaunch => {
                Self::end_token_launch(program_id, accounts)
            }
        }
    } 

    fn transfer_tokens<'a>(
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

    fn create_program_account<'a>(
        funding_account: &AccountInfo<'a>,
        pda : &AccountInfo<'a>,
        program_id :  &Pubkey,
        bump_seed : u8

    ) -> ProgramResult
    {
        let data_size = get_state_size();
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

    fn create_token_account<'a>(
        funding_account : &AccountInfo<'a>,
        wallet_account : &AccountInfo<'a>,
        token_mint_account : &AccountInfo<'a>,
        new_token_account : &AccountInfo<'a>,
        token_program_account : &AccountInfo<'a>

    ) -> ProgramResult
    {
        let create_ATA_idx = create_associated_token_account(&funding_account.key, &wallet_account.key,&token_mint_account.key);

        invoke(
            &create_ATA_idx,
            &[funding_account.clone(), new_token_account.clone(), wallet_account.clone(), token_mint_account.clone(), token_program_account.clone()],
        )?;

        Ok(())
    }

    fn close_program_token_account<'a>(
        program_account_info : &AccountInfo<'a>,
        program_token_account_info : &AccountInfo<'a>,
        destination_account_info : &AccountInfo<'a>,
        destination_token_account_info : &AccountInfo<'a>,
        token_program_account_info : &AccountInfo<'a>,
        bump_seed : u8
    ) -> ProgramResult
    {
        // Check the destination token account exists, which it should do if we are the ones that set it up
        if **destination_token_account_info.try_borrow_lamports()? > 0 {
            msg!("Confirmed destination token account is already initialised.");
        }
        else {

            msg!("destination token account should already exist");
            return Err(ProgramError::InvalidAccountData);
        }

        // And check that we haven't already closed out the program token account
        let program_token_account_lamports = **program_token_account_info.try_borrow_lamports()?;
        if program_token_account_lamports > 0 {
            msg!("Confirmed program token account is still initialised.");
        }
        else {

            msg!("program's token account already closed");
            return Ok(());
        }

        let program_token_account = spl_token::state::Account::unpack_unchecked(&program_token_account_info.try_borrow_data()?)?;

        msg!("transfer token balance: {}", program_token_account.amount);

        if program_token_account.amount > 0 {
            Self::transfer_tokens(
                program_token_account.amount,
                program_token_account_info,
                destination_token_account_info,
                program_account_info,
                token_program_account_info,
                bump_seed
            )?;
        }

        msg!("close account and transfer SOL balance: {}", program_token_account_lamports);

        let close_token_account_idx = spl_token::instruction::close_account(
            token_program_account_info.key,
            program_token_account_info.key, 
            destination_account_info.key, 
            program_account_info.key, 
            &[]
        )?;

        invoke_signed(
            &close_token_account_idx,
            &[program_token_account_info.clone(), destination_account_info.clone(), program_account_info.clone()],
            &[&[b"token_account", &[bump_seed]]]
        )?;

        Ok(())
    }

    // functions to calculate expected public keys
    fn get_expected_daoplays_key() -> Pubkey
    {
        Pubkey::from_str("2BLkynLAWGwW58SLDAnhwsoiAuVtzqyfHKA3W3MJFwEF").unwrap()
    }

    fn get_expected_token_mint_key() -> Pubkey
    {
        Pubkey::from_str("CisHceikLeKxYiUqgDVduw2py2GEK71FTRykXGdwf22h").unwrap()
    }

    fn get_expected_supporters_token_mint_key() -> Pubkey
    {
        Pubkey::from_str("6tnMgdJsWobrWYfPTa1j8pniYL9YR5M6UVbWrxGcvhkK").unwrap()
    }

    fn get_expected_daoplays_token_key() -> Pubkey
    {
        get_associated_token_address(
            &Self::get_expected_daoplays_key(), 
            &Self::get_expected_token_mint_key()
        )
    }

    fn get_expected_daoplays_supporters_token_key() -> Pubkey
    {
        get_associated_token_address(
            &Self::get_expected_daoplays_key(), 
            &Self::get_expected_supporters_token_mint_key()
        )
    }

    fn get_pda_bump() -> u8
    {
        255
    }

    fn get_expected_program_address_key() -> (Pubkey, u8)
    {
        let program_id = Pubkey::from_str("BHJ8pK9WFHad1dEds631tFE6qWQgX48VbwWTSqiwR54Y").unwrap();
        let program_address = Pubkey::create_program_address(&[b"token_account", &[Self::get_pda_bump()]], &program_id).unwrap();

        (program_address, Self::get_pda_bump())
    }

    fn get_expected_program_token_key() -> Pubkey
    {
        get_associated_token_address(
            &Self::get_expected_program_address_key().0, 
            &Self::get_expected_token_mint_key()
        )
    }

    fn get_expected_program_supporters_token_key() -> Pubkey
    {

        get_associated_token_address(
            &Self::get_expected_program_address_key().0, 
            &Self::get_expected_supporters_token_mint_key()
        )
    }

    fn init_token_launch(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        metadata : InitMeta
    ) ->ProgramResult 
    {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed eight accounts, get them all first and then check their value is as expected
        let funding_account_info = next_account_info(account_info_iter)?;
        let program_derived_account_info = next_account_info(account_info_iter)?;
        let token_source_account_info = next_account_info(account_info_iter)?;
        let program_token_account_info = next_account_info(account_info_iter)?;
        let token_mint_account_info = next_account_info(account_info_iter)?;

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
        if funding_account_info.key != &Self::get_expected_daoplays_key() {
            msg!("expected first account to be a daoplays account  {}", Self::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the second account is the program derived address which we can verify with find_program_address
        let (expected_pda_key, bump_seed) = Self::get_expected_program_address_key();
         
        if program_derived_account_info.key != &expected_pda_key {
            msg!("expected second account to be PDA {}", expected_pda_key);
            return Err(ProgramError::InvalidAccountData);
        }
  
        // the third account is the source of the tokens which we can verify with get_associated_token_address
        if token_source_account_info.key != &Self::get_expected_daoplays_token_key() {
            msg!("expected third account to be the funder's token account {}", Self::get_expected_daoplays_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the fourth account is the program's token account
        if program_token_account_info.key != &Self::get_expected_program_token_key() {
            msg!("expected fourth account to be the program's token account {}", Self::get_expected_program_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the fifth account is the mint address for the token
        if token_mint_account_info.key != &Self::get_expected_token_mint_key() {
            msg!("expected fifth account to be the token's mint account {}", Self::get_expected_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the sixth account is the source of the supporter tokens
        if supporters_token_source_account_info.key != &Self::get_expected_daoplays_supporters_token_key() {
            msg!("expected sixth account to be the funder's supporter token account {}", Self::get_expected_daoplays_supporters_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the seventh account is the program's supporters token account
        if program_supporters_token_account_info.key != &Self::get_expected_program_supporters_token_key() {
            msg!("expected seventh account to be the program's supporters token account {}", Self::get_expected_program_supporters_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eighth account is the mint address for the supporter token
        if supporters_token_mint_account_info.key != &Self::get_expected_supporters_token_mint_key() {
            msg!("expected eighth account to be the supporter token's mint account {}", Self::get_expected_supporters_token_mint_key());
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
        

        // Check if the account has already been initialized
        if **program_derived_account_info.try_borrow_lamports()? > 0 {

            msg!("This account is already initialized. skipping");
        }
        else {
            
            msg!("creating PDA");

            Self::create_program_account(
                funding_account_info,
                program_derived_account_info,
                program_id,
                bump_seed
            )?;
        }

       
        if **program_token_account_info.try_borrow_lamports()? > 0 {
            msg!("Programs token account is already initialised. skipping");
        }
        else {

            msg!("creating program's token account");

            Self::create_token_account(
                funding_account_info,
                program_derived_account_info,
                token_mint_account_info,
                program_token_account_info,
                token_program_account_info
            )?;
        }

        if **program_supporters_token_account_info.try_borrow_lamports()? > 0 {
            msg!("Programs supporter token account is already initialised. skipping");
        }
        else {

            msg!("creating program's supporter token account");

            Self::create_token_account(
                funding_account_info,
                program_derived_account_info,
                supporters_token_mint_account_info,
                program_supporters_token_account_info,
                token_program_account_info
            )?;
        }

        Self::transfer_tokens(
            metadata.amount,
            token_source_account_info,
            program_token_account_info,
            funding_account_info,
            token_program_account_info,
            bump_seed
    
        )?;

        Self::transfer_tokens(
            metadata.supporter_amount,
            supporters_token_source_account_info,
            program_supporters_token_account_info,
            funding_account_info,
            token_program_account_info,
            bump_seed
    
        )?;


        Ok(())

    }

    fn join_token_launch(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        meta: JoinMeta
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

        // get the accounts
        let account_info_iter = &mut accounts.iter();

        let joiner_account_info = next_account_info(account_info_iter)?;
        let joiner_token_account_info = next_account_info(account_info_iter)?;
        let joiner_supporters_token_account_info = next_account_info(account_info_iter)?;
       
        let program_data_account_info = next_account_info(account_info_iter)?;
        let program_token_account_info = next_account_info(account_info_iter)?;
        let program_supporters_token_account_info = next_account_info(account_info_iter)?;
        
        let charity_account_info = next_account_info(account_info_iter)?;
        let daoplays_account_info = next_account_info(account_info_iter)?;

        let token_mint_account_info = next_account_info(account_info_iter)?;
        let supporters_token_mint_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;


        // now check all the accounts
        // the joiners account should be the signer
        if !joiner_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // the second account should be the joiners associated token account
        let expected_joiner_token_key = get_associated_token_address(
            &joiner_account_info.key, 
            &token_mint_account_info.key
        );

        if joiner_token_account_info.key != &expected_joiner_token_key
        { 
            msg!("expected second account to be the joiner's associated token account {}", expected_joiner_token_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the third account should be the joiners supporter associated token account
        let expected_joiner_supporters_token_key = get_associated_token_address(
            &joiner_account_info.key, 
            &supporters_token_mint_account_info.key
        );

        if joiner_supporters_token_account_info.key != &expected_joiner_supporters_token_key
        { 
            msg!("expected third account to be the joiner's supporter associated token account {}", expected_joiner_supporters_token_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the fourth account should be the programs derived account
        let (expected_pda_key, bump_seed) = Self::get_expected_program_address_key();

        if program_data_account_info.key != &expected_pda_key
        { 
            msg!("expected fourth account to be the programs derived account {}", expected_pda_key);
            return Err(ProgramError::InvalidAccountData); 
        }

        // the fifth account should be the programs token address
        if program_token_account_info.key != &Self::get_expected_program_token_key()
        { 
            msg!("expected fifth account to be the programs token account {}", Self::get_expected_program_token_key());
            return Err(ProgramError::InvalidAccountData); 
        }

        // the sixth account should be the programs token address
        if program_supporters_token_account_info.key != &Self::get_expected_program_supporters_token_key()
        { 
            msg!("expected sixth account to be the programs supporter token account {}", Self::get_expected_program_supporters_token_key());
            return Err(ProgramError::InvalidAccountData); 
        }

        // the seventh account is the charity SOL address, which we can check with the map
        let expected_charity_key = Pubkey::from_str(charity_key_map[meta.charity]).unwrap();

        if charity_account_info.key != &expected_charity_key
        {
            msg!("expected fifth account to be the chosen charities address {}", expected_charity_key);
            return Err(ProgramError::InvalidAccountData);
        }

        // the eighth account is the daoplays SOL address
         if daoplays_account_info.key != &Self::get_expected_daoplays_key()
        {
            msg!("expected sixth account to be the daoplays address {}", Self::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the ninth account is the mint address for the token
        if token_mint_account_info.key != &Self::get_expected_token_mint_key()
        {
            msg!("expected ninth account to be the token mint address {}", Self::get_expected_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the tenth account is the mint address for the supporters token
        if supporters_token_mint_account_info.key != &Self::get_expected_supporters_token_mint_key()
        {
            msg!("expected tenth account to be the token mint address {}", Self::get_expected_supporters_token_mint_key());
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

        // the thirteenth and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected thirteenth account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }
 
        // check if we need to create the joiners token account
        if **joiner_token_account_info.try_borrow_lamports()? > 0 {
            msg!("Users token account is already initialised.");

        }

        else {

            msg!("creating user's token account");

            Self::create_token_account(
                joiner_account_info,
                joiner_account_info,
                token_mint_account_info,
                joiner_token_account_info,
                token_program_account_info
            )?;
        }

        // check that this transaction is valid:
        // i) total amount should exceed the minimum
        // ii) joiner should not already have tokens
        // iii) program should have enough spare tokens

        
        msg!("Transfer {} {}", meta.amount_charity, meta.amount_dao);
        msg!("Balance {}", joiner_account_info.try_borrow_lamports()?);

        let min_amount : u64 = 100000;
        if meta.amount_charity + meta.amount_dao < min_amount {
            msg!("Amount paid is less than the minimum of 0.0001 SOL");
            return Err(ProgramError::InvalidArgument);
        }

        let program_token_account = spl_token::state::Account::unpack_unchecked(&program_token_account_info.try_borrow_data()?)?;
        let program_supporters_token_account = spl_token::state::Account::unpack_unchecked(&program_supporters_token_account_info.try_borrow_data()?)?;
        let joiner_token_account = spl_token::state::Account::unpack_unchecked(&joiner_token_account_info.try_borrow_data()?)?;

        msg!("token balances: {} {} {}", program_token_account.amount, program_supporters_token_account.amount, joiner_token_account.amount);

        if joiner_token_account.amount > 0 {
            msg!("Tokens already present in joiners account, thank you for taking part!");
            return Err(ProgramError::InvalidAccountData);
        }

        // get the data stored in the program account to access current state
        let mut current_state = TokenLaunchData::try_from_slice(&program_data_account_info.data.borrow()[..])?;

        // calculate the current average to see if this individual has paid more
        let current_average = current_state.paid_total / current_state.n_donations;
        let total_paid = meta.amount_charity + meta.amount_dao;
        let mut token_launch_amount : u64 = 1000;

        let mut supporter = false;
        // if they have then they get double!
        if total_paid > current_average {
            msg!("Thank you for paying over the average price!");

            token_launch_amount = 2000;
            supporter =  true;
        }
        
        // check if there are the required number of tokens remaining
        if program_token_account.amount < token_launch_amount {
            msg!("Insufficient tokens remaining in token launch");
            return Err(ProgramError::InvalidArgument);
        }

        // if we have made it this far the transaction we can try transferring the SOL
        invoke(
            &system_instruction::transfer(joiner_account_info.key, charity_account_info.key, meta.amount_charity),
            &[joiner_account_info.clone(), charity_account_info.clone()],
        )?;

        invoke(
            &system_instruction::transfer(joiner_account_info.key, daoplays_account_info.key, meta.amount_dao),
            &[joiner_account_info.clone(), daoplays_account_info.clone()],
        )?;

        // and finally transfer the tokens
        Self::transfer_tokens(
            token_launch_amount,
            program_token_account_info,
            joiner_token_account_info,
            program_data_account_info,
            token_program_account_info,
            bump_seed
        )?;

        if supporter && program_supporters_token_account.amount >= 1 {

            // check if we need to create the joiners supporter token account
            if **joiner_supporters_token_account_info.try_borrow_lamports()? > 0 {
                msg!("Users supporter token account is already initialised.");

            }

            else {

                msg!("creating user's supporter token account");

                Self::create_token_account(
                    joiner_account_info,
                    joiner_account_info,
                    supporters_token_mint_account_info,
                    joiner_supporters_token_account_info,
                    token_program_account_info
                )?;
            }


            Self::transfer_tokens(
                1,
                program_supporters_token_account_info,
                joiner_supporters_token_account_info,
                program_data_account_info,
                token_program_account_info,
                bump_seed
            )?;
    
        }

        // update the data

        let charity_index = charity_index_map[meta.charity];

        current_state.charity_totals[charity_index] += meta.amount_charity;
        current_state.donated_total += meta.amount_charity;
        current_state.paid_total += total_paid;
        current_state.n_donations += 1;

        msg!("Updating current state: {} {} {} {}", current_state.charity_totals[charity_index], current_state.donated_total, current_state.paid_total,  current_state.n_donations);

        current_state.serialize(&mut &mut program_data_account_info.data.borrow_mut()[..])?;


        Ok(())
    }

    fn end_token_launch(
        _program_id: &Pubkey,
        accounts: &[AccountInfo]
    ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        let daoplays_account_info = next_account_info(account_info_iter)?;
        let daoplays_token_account_info = next_account_info(account_info_iter)?;
        let daoplays_supporters_token_account_info = next_account_info(account_info_iter)?;

        let program_account_info = next_account_info(account_info_iter)?;
        let program_token_account_info = next_account_info(account_info_iter)?;
        let program_supporters_token_account_info = next_account_info(account_info_iter)?;

        let token_mint_account_info = next_account_info(account_info_iter)?;
        let supporters_token_mint_account_info = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;


        // the first account should be the funding account and should be a signer
        if !daoplays_account_info.is_signer {
            msg!("expected first account as signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // only we should be able to call this function
        if daoplays_account_info.key != &Self::get_expected_daoplays_key() {
            msg!("expected first account to be a daoplays account  {}", Self::get_expected_daoplays_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the second account should be the daoplays token account we want to transfer back to
        if daoplays_token_account_info.key != &Self::get_expected_daoplays_token_key()
        {
            msg!("expected second account to be a daoplays token account  {}", Self::get_expected_daoplays_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the third account should be the daoplays supporters token account we want to transfer back to
        if daoplays_supporters_token_account_info.key != &Self::get_expected_daoplays_supporters_token_key()
        {
            msg!("expected third account to be a daoplays supporters token account  {}", Self::get_expected_daoplays_supporters_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the fourth account should be the program's derived address, which we can verify using find_program_address
        let (expected_program_account_key, bump_seed) = Self::get_expected_program_address_key();

        if program_account_info.key != &expected_program_account_key
        {
            msg!("expected fourth account to be a program's derived account  {}", expected_program_account_key);
            return Err(ProgramError::InvalidAccountData);
        }
         
        // the fifth account should be the program's token account
        if program_token_account_info.key != &Self::get_expected_program_token_key()
        {
            msg!("expected fifth account to be a program's token account  {}", Self::get_expected_program_token_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the sixth account should be the programs supporters token account
        if program_supporters_token_account_info.key != &Self::get_expected_program_supporters_token_key()
        {
            msg!("expected sixth account to be a daoplays supporters token account  {}", Self::get_expected_program_supporters_token_key());
            return Err(ProgramError::InvalidAccountData);
        }
        
        // the seventh account is the mint address for the token
        if token_mint_account_info.key != &Self::get_expected_token_mint_key() {
            msg!("expected seventh account to be the token's mint account {}", Self::get_expected_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eighth account is the mint address for the supporters token
        if supporters_token_mint_account_info.key != &Self::get_expected_supporters_token_mint_key() {
            msg!("expected eighth account to be the token's mint account {}", Self::get_expected_supporters_token_mint_key());
            return Err(ProgramError::InvalidAccountData);
        }

        // the ninth should be the token program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected ninth account to be the token program");
            return Err(ProgramError::InvalidAccountData);
        }

        // the tenth and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected tenth account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // first close out the main token account
        Self::close_program_token_account(
            program_account_info,
            program_token_account_info,
            daoplays_account_info,
            daoplays_token_account_info,
            token_program_account_info,
            bump_seed
        )?;

        // now do the same thing for the supporters tokens
        Self::close_program_token_account(
            program_account_info,
            program_supporters_token_account_info,
            daoplays_account_info,
            daoplays_supporters_token_account_info,
            token_program_account_info,
            bump_seed
        )?;

        Ok(())

    }
}