use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::RNGError::InvalidInstruction;
use crate::state::{RNGMeta};




#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum RNGInstruction {

    GenerateRandom {
        metadata: RNGMeta
    }
}

impl RNGInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::GenerateRandom {
                metadata: RNGMeta::try_from_slice(&rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }
}