use borsh::{BorshDeserialize, BorshSerialize};
use crate::state::{get_score_meta_size, ScoreMeta};
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
use crate::{instruction::UnityInstruction};

pub struct Processor;
impl Processor {
    
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        let instruction = UnityInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            
            UnityInstruction::UploadScore {metadata} => {
                Self::upload_score(program_id, accounts, metadata)
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

    fn upload_score(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        metadata : ScoreMeta
    ) -> ProgramResult
    {

        let account_info_iter = &mut accounts.iter();

        // This function expects to be passed six accounts, get them all first and then check their value is as expected
        let player_account_info = next_account_info(account_info_iter)?;

        let player_data_account_info = next_account_info(account_info_iter)?;

        let _system_program_account_info = next_account_info(account_info_iter)?;


        if !player_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (expected_data_account,bump_seed) = Pubkey::find_program_address(&[&player_account_info.key.to_bytes()], &program_id);

        // the second account is the program data account
        if player_data_account_info.key != &expected_data_account
        {
            msg!("expected second account to be the program data account {}", expected_data_account);
            return Err(ProgramError::InvalidAccountData);
        }

        Self::create_program_account(player_account_info,
            player_data_account_info,
            program_id,
            bump_seed,
            state::get_score_meta_size(),
            &player_account_info.key.to_bytes()
        )?;

        let mut player_data = state::ScoreMeta::try_from_slice(&player_data_account_info.data.borrow())?;
        
        if metadata.high_score > player_data.high_score {
            player_data.high_score = metadata.high_score;
            player_data.serialize(&mut &mut player_data_account_info.data.borrow_mut()[..])?;
        }

        Ok(())
    }
   
}