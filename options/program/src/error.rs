use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;
#[derive(Error, Clone, Debug, Eq, PartialEq)]
pub enum OptionsError {
    // past deadlin
    #[error("Past Deadline")]
    PastDeadline,
    #[error("Before Deadline")]
    BeforeDeadline,
}

impl PrintProgramError for OptionsError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<OptionsError> for ProgramError {
    fn from(e: OptionsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for OptionsError {
    fn type_of() -> &'static str {
        "Options Error"
    }
}
