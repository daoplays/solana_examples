use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::error::NewError::InvalidInstruction;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CreateMeta {
    pub extensions: u8,
    pub transfer_fee_bp: u16,
    pub transfer_fee_max: u64,
    pub interest_rate: i16,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TokenInstruction {
    CreateToken { metadata: CreateMeta },
}

impl TokenInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::CreateToken {
                metadata: CreateMeta::try_from_slice(&rest)?,
            },

            _ => return Err(InvalidInstruction.into()),
        })
    }
}
