use thiserror::Error;
use solana_program::program_error::ProgramError;


#[derive(Error, Debug, Copy, Clone)]
pub enum NewError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
    #[error("Team Account Not Created")]
    TeamAccountNotCreated,
    #[error("User Doesn't own Team Token")]
    NoTeamTokens,
    #[error("Team Name too long")]
    InvalidTeamName,
    #[error("Invalid Token Mint")]
    InvalidTokenMint,
    #[error("Team Already Exists")]
    TeamAlreadyExists,
}

impl From<NewError> for ProgramError {
    fn from(e: NewError) -> Self {
        ProgramError::Custom(e as u32)
    }
}