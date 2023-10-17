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

/// Errors that may be returned by the interface.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum TransferHookError {
    /// Incorrect account provided
    #[error("Incorrect account provided")]
    IncorrectAccount,
    /// Mint has no mint authority
    #[error("Mint has no mint authority")]
    MintHasNoMintAuthority,
    /// Incorrect mint authority has signed the instruction
    #[error("Incorrect mint authority has signed the instruction")]
    IncorrectMintAuthority,
    /// Program called outside of a token transfer
    #[error("Program called outside of a token transfer")]
    ProgramCalledOutsideOfTransfer,
}
impl From<TransferHookError> for ProgramError {
    fn from(e: TransferHookError) -> Self {
        ProgramError::Custom(e as u32)
    }
}