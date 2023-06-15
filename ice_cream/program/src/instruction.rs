use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::NewError::InvalidInstruction;
use crate::state::{CreateMeta};



#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum IceCreamInstruction {

    InitProgram,

    CreateTeam {
        metadata: CreateMeta
    },

    CreateTeamLookup {
        metadata: CreateMeta
    },

    Eat {
        metadata: CreateMeta
    }
}

impl IceCreamInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {

            0 => Self::InitProgram,
            1 => Self::CreateTeam {
                metadata: CreateMeta::try_from_slice(&rest)?
            },
            2 => Self::CreateTeamLookup {
                metadata: CreateMeta::try_from_slice(&rest)?
            },
            3 => Self::Eat {
                metadata: CreateMeta::try_from_slice(&rest)?
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }
}