use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::NewError::InvalidInstruction;
use crate::state::{ScoreMeta};



#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum UnityInstruction {

    UploadScore {
        metadata: ScoreMeta
    }
}

impl UnityInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {

            0 => Self::UploadScore {
                metadata: ScoreMeta::try_from_slice(&rest)?
            },
            
            _ => return Err(InvalidInstruction.into()),
        })
    }
}