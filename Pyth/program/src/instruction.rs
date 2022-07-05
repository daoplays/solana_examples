use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::RNGError::InvalidInstruction;



#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum RNGInstruction {

    GenerateSeed
}

impl RNGInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, _rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::GenerateSeed,
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
