use thiserror::Error;
use solana_program::program_error::ProgramError;


#[derive(Error, Debug, Copy, Clone)]
pub enum NewError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction
}

impl From<NewError> for ProgramError {
    fn from(e: NewError) -> Self {
        ProgramError::Custom(e as u32)
    }
}