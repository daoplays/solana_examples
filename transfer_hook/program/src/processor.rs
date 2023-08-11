use crate::accounts;
use crate::state;
use crate::utils;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::rent,
};

use spl_token::{instruction, state::Mint};

use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};

use crate::error::NewError;

use crate::instruction::TransferHookInstruction;

pub struct Processor;
impl Processor {
    /// Processes an [Instruction](enum.Instruction.html).
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = TransferHookInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            TransferHookInstruction::Execute { amount } => {
                msg!("Instruction: Execute");
                Self::process_execute(program_id, accounts, amount)
            }
            TransferHookInstruction::InitializeExtraAccountMetas => {
                msg!("Instruction: InitializeExtraAccountMetas");
                Self::process_initialize_extra_account_metas(program_id, accounts)
            }
        }
    }

    /// Processes an [Execute](enum.TransferHookInstruction.html) instruction.
    pub fn process_execute(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        _amount: u64,
    ) -> ProgramResult {
        Ok(())
    }

    pub fn process_initialize_extra_account_metas(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
    ) -> ProgramResult {
        Ok(())
    }
}
