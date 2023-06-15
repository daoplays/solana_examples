use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::state::Charity;
use crate::error::DaoPlaysError::InvalidInstruction;


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct BidData {
    // the amount in lamports that will be donated to charity
    pub amount_charity : u64,
    // the amount in lamports being paid to the developers
    pub amount_dao : u64,
    // the chosen charity
    pub charity : Charity
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct InitMeta {
    // the amount of DPTTs to be sent to the program
    pub amount : u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum DaoPlaysInstruction {

    CreateDataAccount {
        metadata : InitMeta
    },

    PlaceBid {
        // the price to bid in lamports
        bid_data: BidData
    },

    SelectWinners,

    SendTokens
}

impl DaoPlaysInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::CreateDataAccount  {
                metadata: InitMeta::try_from_slice(&rest)?,
            },
            1 => Self::PlaceBid{
                bid_data: BidData::try_from_slice(&rest)?,
            },
            2 => Self::SelectWinners,
            3 => Self::SendTokens,
            _ => return Err(InvalidInstruction.into()),
        })
    }
}