use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::RNGError::InvalidInstruction;
use crate::state::{SeedMeta};


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum SeedInstruction {

    GenerateSeed{
        metadata: SeedMeta
    }
}

impl SeedInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::GenerateSeed{
                metadata: SeedMeta::try_from_slice(&rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
