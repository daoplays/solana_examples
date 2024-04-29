use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::error::NewError::InvalidInstruction;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CreateMeta {
    pub name: String,
    pub uri: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TokenInstruction {
    CreateCollection { args: CreateMeta },
    CreateToken { args: CreateMeta },
    TransferToken,
}

impl TokenInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::CreateCollection {
                args: CreateMeta::try_from_slice(&rest)?,
            },
            1 => Self::CreateToken {
                args: CreateMeta::try_from_slice(&rest)?,
            },
            2 => Self::TransferToken,
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
