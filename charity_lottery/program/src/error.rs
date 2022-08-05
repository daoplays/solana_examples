use thiserror::Error;
use solana_program::program_error::ProgramError;


#[derive(Error, Debug, Copy, Clone)]
pub enum DaoPlaysError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,

    #[error("Invalid bid amount for button press")]
    InvalidButtonBid
}

impl From<DaoPlaysError> for ProgramError {
    fn from(e: DaoPlaysError) -> Self {
        ProgramError::Custom(e as u32)
    }
}