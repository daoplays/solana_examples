use borsh::{BorshDeserialize, BorshSerialize};
use crate::state::{CreateMeta};
use crate::state;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,msg,
    program_error::ProgramError,
    program::invoke_signed,
    program::invoke,
    sysvar::rent,
    program_pack::Pack
};

use spl_token::{instruction, state::{Mint}};

use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};

use crate::error::NewError;
use crate::{instruction::IceCreamInstruction};

pub struct Processor;
impl Processor {
    
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        let instruction = IceCreamInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            IceCreamInstruction::InitProgram => {
                Self::init_program(program_id, accounts)
            },
            IceCreamInstruction::CreateTeam {metadata} => {
                Self::create_team(program_id, accounts, metadata)
            },
            IceCreamInstruction::CreateTeamLookup {metadata} => {
                Self::create_team_lookup(program_id, accounts, metadata)
            },
            IceCreamInstruction::Eat {metadata} => {
                Self::eat(program_id, accounts, metadata)
            }

        }
    } 

    pub fn create_program_account<'a>(
        funding_account: &AccountInfo<'a>,
        pda : &AccountInfo<'a>,
        program_id :  &Pubkey,
        bump_seed : u8,
        data_size : usize,
        seed : &[u8]
    
    ) -> ProgramResult
    {
    
         // Check if the account has already been initialized
         if **pda.try_borrow_lamports()? > 0 {
            msg!("This account is already initialized. skipping");
            return Ok(());
        }
    
        msg!("Creating program derived account");
    
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
            &[&[seed, &[bump_seed]]]
        )?;
    
        Ok(())
    }

    fn create_mint_account<'a>(
        funding_account: &AccountInfo<'a>,
        mint_account: &AccountInfo<'a>,
        new_token_account: &AccountInfo<'a>,
        token_program: &AccountInfo<'a>,
        rent_program: &AccountInfo<'a>
    ) -> ProgramResult
    {
    
        let mint_rent = rent::Rent::default().minimum_balance(Mint::LEN);
    
        let ix = solana_program::system_instruction::create_account(
            funding_account.key,
            mint_account.key,
            mint_rent,
            Mint::LEN as u64,
            token_program.key,
        );
    
        // Sign and submit transaction
        invoke(
            &ix,
            &[funding_account.clone(), mint_account.clone()]
        )?;

       let mint_idx = instruction::initialize_mint(
            token_program.key,
            mint_account.key,
            funding_account.key,
            None,
            0
        ).unwrap();

        // Sign and submit transaction
        invoke(
            &mint_idx,
            &[token_program.clone(), mint_account.clone(), funding_account.clone(), rent_program.clone()]
        )?;

        // create the ATA
        let create_ata_idx = create_associated_token_account(&funding_account.key, &funding_account.key,&mint_account.key);

        invoke(
            &create_ata_idx,
            &[funding_account.clone(), new_token_account.clone(), funding_account.clone(), mint_account.clone(), token_program.clone()],
        )?;

        // and finally mint the user one token on their behalf so they can access the game right away
        let mint_to_idx = instruction::mint_to(
            token_program.key,
            mint_account.key,
            new_token_account.key,
            funding_account.key,
            &[funding_account.key],
            1
        ).unwrap();

        invoke(
            &mint_to_idx,
            &[token_program.clone(), mint_account.clone(), new_token_account.clone(), funding_account.clone()]
        )?;
   
        Ok(())
    }

    fn create_team(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        metadata : CreateMeta
    ) ->ProgramResult 
    {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed nine accounts, get them all first and then check their value is as expected
        let funding_account_info = next_account_info(account_info_iter)?;
        let token_mint_account_info = next_account_info(account_info_iter)?;
        let new_token_account = next_account_info(account_info_iter)?;

        let program_data_account = next_account_info(account_info_iter)?;
        let team_data_account = next_account_info(account_info_iter)?;

        let token_program_account_info = next_account_info(account_info_iter)?;
        let associated_token_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;
        let rent_account_info = next_account_info(account_info_iter)?;

        if !funding_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let expected_token_pubkey = get_associated_token_address(
            &funding_account_info.key, 
            &token_mint_account_info.key
        );

        // the third account is the user's token account
        if new_token_account.key != &expected_token_pubkey
        {
            msg!("expected third account to be the user token account {}", expected_token_pubkey);
            return Err(ProgramError::InvalidAccountData);
        }

        let (expected_data_account, _bump_seed) = Pubkey::find_program_address(&[b"data_account"], &program_id);

        // the fourth account is the program data account
        if program_data_account.key != &expected_data_account
        {
            msg!("expected fifth account to be the data account {}", expected_data_account);
            return Err(ProgramError::InvalidAccountData);
        }

        let (expected_team_account, team_bump_seed) = Pubkey::find_program_address(&[metadata.team_name.as_bytes()], &program_id);

        // the fifth account is the team's data account
        if team_data_account.key != &expected_team_account
        {
            msg!("expected fifth account to be the team data account {}", expected_team_account);
            return Err(ProgramError::InvalidAccountData);
        }

        // check if the team already has an account
        if **team_data_account.try_borrow_lamports()? > 0 {
            msg!("This team already exists");
            return Err(ProgramError::from(NewError::TeamAlreadyExists));
        }

        // if not this is a new team, so check that the team name is valid
        let team_name_bytes =  metadata.team_name.as_bytes();
        let name_len = team_name_bytes.len();

        if name_len >= 256 {
            msg!("team name {} is too long", metadata.team_name);
            return Err(ProgramError::from(NewError::InvalidTeamName));
        }

        // the sixth account is the token_program
        if token_program_account_info.key != &spl_token::id() {
            msg!("expected sixth account to be the token program {}", spl_token::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the seventh account is the associated_token_program
        if associated_token_account_info.key != &spl_associated_token_account::id() {
            msg!("expected twelfth account to be the associated token program {}", spl_associated_token_account::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the eighth account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected eighth account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the ninth and final account is the sysvar rent account
        if rent_account_info.key != &solana_program::sysvar::rent::id() {
            msg!("expected eighth account to be the rent program {}", solana_program::sysvar::rent::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // if the program has been passed an existing token mint, it must satisfy some conditions which we check below
        if **token_mint_account_info.try_borrow_lamports()? > 0 {

            // first get the mint account data
            let mint_account_state = spl_token::state::Mint::unpack_unchecked(&token_mint_account_info.try_borrow_data()?)?;
            
            // if the mint account has not been initialised return an error
            if !mint_account_state.is_initialized {
                msg!("mint account {} has not been initialized", token_mint_account_info.key.to_string());
                return Err(ProgramError::from(NewError::InvalidTokenMint));
            }

            // if the supply is 0 then this isn't a valid token
            if !mint_account_state.supply == 0 {
                msg!("mint account {} has zero supply", token_mint_account_info.key.to_string());
                return Err(ProgramError::from(NewError::InvalidTokenMint));
            }

            // if decimals isn't zero this isn't a valid token
            if mint_account_state.decimals != 0 {
                msg!("mint account {} has invalid decimal places ({} != 0)", token_mint_account_info.key.to_string(), mint_account_state.decimals);
                return Err(ProgramError::from(NewError::InvalidTokenMint));
            }

            // finally if the mint authority is not Some, and if it doesn't match the funding account, it isn't a valid choice
            if mint_account_state.mint_authority.is_none() {
                msg!("mint account {} has no mint authority", token_mint_account_info.key.to_string());
                return Err(ProgramError::from(NewError::InvalidTokenMint));
            }

            let mint_authority = mint_account_state.mint_authority.unwrap();
            if mint_authority != *funding_account_info.key {
                msg!("mint account {} authority {} is not the funding account {}", token_mint_account_info.key.to_string(), mint_authority.to_string(), funding_account_info.key.to_string());
                return Err(ProgramError::from(NewError::InvalidTokenMint));
            }
        }
        else {

            // if the mint account didn't exist then create it now
            Self::create_mint_account(
                funding_account_info,
                token_mint_account_info,
                new_token_account,
                token_program_account_info,
                rent_account_info
            )?;
        }

        // increment the total number of teams the program knows about, this value will be used to index this team in it's lookup account
        let mut score_data = state::ScoreMeta::try_from_slice(&program_data_account.data.borrow())?;
        score_data.num_teams += 1;
        score_data.serialize(&mut &mut program_data_account.data.borrow_mut()[..])?;


        Self::create_program_account(funding_account_info,
            team_data_account,
            program_id,
            team_bump_seed,
            state::get_team_meta_size(),
            metadata.team_name.as_bytes())?;

        // copy the team name to a byte array
        let mut meta_bytes = [0 as u8 ; 256];
        for i in 0..name_len {
            meta_bytes[i] = team_name_bytes[i];
        }

        let team_meta = state::TeamMeta{team_name : meta_bytes, name_len : name_len as u64, mint_address : *token_mint_account_info.key, score : 0, index : score_data.num_teams};
        
        team_meta.serialize(&mut &mut team_data_account.data.borrow_mut()[..])?;

        Ok(())

    }

    fn create_team_lookup(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        metadata : CreateMeta
    ) ->ProgramResult 
    {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed four accounts, get them all first and then check their value is as expected
        let funding_account_info = next_account_info(account_info_iter)?;
        
        let team_lookup_account = next_account_info(account_info_iter)?;
        let team_data_account = next_account_info(account_info_iter)?;

        let system_program_account_info = next_account_info(account_info_iter)?;

        if !funding_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (expected_team_account, _team_bump_seed) = Pubkey::find_program_address(&[metadata.team_name.as_bytes()], &program_id);

        // the third account is the team's data account
        if team_data_account.key != &expected_team_account
        {
            msg!("expected third account to be the team data account {}", expected_team_account);
            return Err(ProgramError::InvalidAccountData);
        }

        // the fourth and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected fourth account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        // the team's data account must have been created already
        if **team_data_account.try_borrow_lamports()? <= 0 {
            msg!("Team account has not been created yet");
            return Err(ProgramError::from(NewError::TeamAccountNotCreated));
        }

        let team_data = state::TeamMeta::try_from_slice(&team_data_account.data.borrow())?;
        let index = team_data.index;

        let (expected_team_lookup_account, team_lookup_bump_seed) = Pubkey::find_program_address(&[&index.to_le_bytes()], &program_id);

        // the second account is the team lookup account
        if team_lookup_account.key != &expected_team_lookup_account
        {
            msg!("expected second account to be the team data account {}", expected_team_lookup_account);
            return Err(ProgramError::InvalidAccountData);
        }

        Self::create_program_account(funding_account_info,
            team_lookup_account,
            program_id,
            team_lookup_bump_seed,
            state::get_team_account_meta_size(),
            &index.to_le_bytes())?;
        
        // the lookup just stores the address of this teams data account
        let team_account_meta = state::TeamAccountMeta{team_account : *team_data_account.key};

        team_account_meta.serialize(&mut &mut team_lookup_account.data.borrow_mut()[..])?;

        Ok(())

    }


    fn init_program(
        program_id: &Pubkey,
        accounts: &[AccountInfo]
    ) -> ProgramResult
    {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed three accounts, get them all first and then check their value is as expected
        let funding_account_info = next_account_info(account_info_iter)?;
        let program_data_account = next_account_info(account_info_iter)?;

        let system_program_account_info = next_account_info(account_info_iter)?;

        if !funding_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (expected_data_account,bump_seed) = Pubkey::find_program_address(&[b"data_account"], &program_id);

        // the second account is the program data account
        if program_data_account.key != &expected_data_account
        {
            msg!("expected second account to be the program data account {}", expected_data_account);
            return Err(ProgramError::InvalidAccountData);
        }

        // the third and final account is the system_program
        if system_program_account_info.key != &solana_program::system_program::id() {
            msg!("expected third account to be the system program {}", solana_program::system_program::id());
            return Err(ProgramError::InvalidAccountData);
        }

        Self::create_program_account(funding_account_info,
            program_data_account,
            program_id,
            bump_seed,
            state::get_score_meta_size(),
            "data_account".as_bytes()
        )?;

        Ok(())
    }

    fn eat(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        metadata : CreateMeta
    ) -> ProgramResult
    {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed six accounts, get them all first and then check their value is as expected
        let funding_account_info = next_account_info(account_info_iter)?;

        let token_mint_account = next_account_info(account_info_iter)?;
        let user_token_account = next_account_info(account_info_iter)?;

        let program_data_account = next_account_info(account_info_iter)?;
        let team_data_account = next_account_info(account_info_iter)?;

        let associated_token_account_info = next_account_info(account_info_iter)?;


        if !funding_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (expected_data_account, _bump_seed) = Pubkey::find_program_address(&[b"data_account"], &program_id);

        // the fourth account is the program data account
        if program_data_account.key != &expected_data_account
        {
            msg!("expected fourth account to be the program data account {}", expected_data_account);
            return Err(ProgramError::InvalidAccountData);
        }

        let (expected_team_account, _team_bump_seed) = Pubkey::find_program_address(&[metadata.team_name.as_bytes()], &program_id);
        
        // the fifth account is the team data account
        if team_data_account.key != &expected_team_account
        {
            msg!("expected fifth account to be the team data account {}", expected_team_account);
            return Err(ProgramError::InvalidAccountData);
        }

        // the sixth account is the associated token program
        // the seventh account is the associated_token_program
        if associated_token_account_info.key != &spl_associated_token_account::id() {
            msg!("expected sixth account to be the associated token program {}", spl_associated_token_account::id());
            return Err(ProgramError::InvalidAccountData);
        }

         // the team's data account must have been created already
         if **team_data_account.try_borrow_lamports()? <= 0 {
            msg!("Team account has not been created yet");
            return Err(ProgramError::from(NewError::TeamAccountNotCreated));
        }

        // get the expected mint address for this team from the data account
        let mut team_data = state::TeamMeta::try_from_slice(&team_data_account.data.borrow())?;
        let expected_team_mint_key = team_data.mint_address;

        // the second account is the team's token mint
        if token_mint_account.key != &expected_team_mint_key
        {
            msg!("expected second account to be the team mint account {}", expected_team_account);
            return Err(ProgramError::InvalidAccountData);
        }

        let expected_token_pubkey = get_associated_token_address(
            &funding_account_info.key, 
            &token_mint_account.key
        );

        // the third account is the user's token account
        if user_token_account.key != &expected_token_pubkey
        {
            msg!("expected third account to be the user token account {}", expected_token_pubkey);
            return Err(ProgramError::InvalidAccountData);
        }

        // this user's token account must have been created already
        if **user_token_account.try_borrow_lamports()? <= 0 {
            msg!("User token account has not been created yet");
            return Err(ProgramError::InvalidAccountData);
        }

        let user_token_account_state = spl_token::state::Account::unpack_unchecked(&user_token_account.try_borrow_data()?)?;

        // check the user has tokens from the team mint
        if user_token_account_state.amount <= 0 {
            msg!("User has no team tokens");
            return Err(ProgramError::from(NewError::NoTeamTokens));
        }


        let mut score_data = state::ScoreMeta::try_from_slice(&program_data_account.data.borrow())?;

        // increment the team's score
        team_data.score += 1;
        team_data.serialize(&mut &mut team_data_account.data.borrow_mut()[..])?;

        // check if the new score is higher than the lowest of the top 10
        let mut min: u64 = u64::MAX;
        let mut min_index : usize = 0;
        let mut present : bool = false;
        for i in 0..10 {
            if team_data.index == score_data.top_ten_teams[i] {
                present = true;
                min_index = i;
                break;
            }
            if score_data.top_ten_scores[i] < min {
                min = score_data.top_ten_scores[i];
                min_index = i;
            }
        }

        if present {
            msg!("Team already present in the top 10! {} for team {}", team_data.score, team_data.index);

            score_data.top_ten_scores[min_index] = team_data.score;
            score_data.top_ten_teams[min_index] = team_data.index;

            score_data.serialize(&mut &mut program_data_account.data.borrow_mut()[..])?;
        }

        if !present && team_data.score > min {

            msg!("New entry in top 10! {} > {} for team {}", team_data.score, min, team_data.index);

            score_data.top_ten_scores[min_index] = team_data.score;
            score_data.top_ten_teams[min_index] = team_data.index;
            
            score_data.serialize(&mut &mut program_data_account.data.borrow_mut()[..])?;

        }

        Ok(())
    }
   
}