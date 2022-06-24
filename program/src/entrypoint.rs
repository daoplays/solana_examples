use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use crate::processor::Processor;

// entrypoint has only one allowed instruction: GenerateRandom
// this will generate 512 random u64's given the method specified in the
// 'method' argument for that instruction (see instruction.rs for more detail)
entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {   
    Processor::process(program_id, accounts, instruction_data)
}