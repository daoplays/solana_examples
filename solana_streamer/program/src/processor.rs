use borsh::{BorshDeserialize};
use crate::state::{ChoiceData};


use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,msg,
    program_error::ProgramError
};

use crate::{instruction::ChoiceInstruction};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        let instruction = ChoiceInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            ChoiceInstruction::MakeChoice { choice_data } => {

                Self::make_choice(program_id, accounts, choice_data)
            }
        }
    } 
    
 
    fn make_choice(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        choice_data: ChoiceData
        ) ->ProgramResult {

        msg!("choice has been made: {:?} {}", choice_data.choice, choice_data.bid_amount);

        Ok(())
    }
}
