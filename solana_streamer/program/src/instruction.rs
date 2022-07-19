use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::RNGError::InvalidInstruction;
use crate::state::{ChoiceData};


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum ChoiceInstruction {

    // MakeChoice expects only one account, the user of the program which should be signed
    MakeChoice {
        choice_data: ChoiceData
    }
}

impl ChoiceInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {       
            0 => Self::MakeChoice {
                choice_data: ChoiceData::try_from_slice(&rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
