use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};
use crate::error::DaoPlaysError::InvalidInstruction;


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct RegisterMeta {
    // the string of the id that contains the users pubkey
    pub tweet_id : u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct HashTagMeta {
    // the string of the id that contains the users pubkey
    pub tweet_id : u64,
    pub hashtag : String
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct UserMeta {
    // the string of the id that contains the users pubkey
    pub user_id : u64
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ErrorMeta {
    // the amount of supporter tokens to be send to the program
    pub error_code : u8
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TokenMeta {
    // the amount of supporter tokens to be sent to the user
    pub amount : u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct HashTagRewardMeta {
    // the amount of supporter tokens to be sent to the user
    pub amount : u64,
    pub tweet_id : u64,
    pub hashtag : String
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TwitterInstruction {

    InitProgram {
        metadata: TokenMeta
    },
    Register {
        metadata : RegisterMeta
    },
    CreateUserAccount,
    NewFollower {
        metadata : UserMeta
    },
    SetError {
        metadata : ErrorMeta
    },
    CheckFollower,
    CheckHashTag {
        metadata : HashTagMeta
    },
    SendTokens {
        metadata : HashTagRewardMeta
    },
    CheckRetweet {
        metadata : HashTagMeta
    },
    SetUserID  {
        metadata : UserMeta
    }
}

impl TwitterInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::InitProgram  {
                metadata: TokenMeta::try_from_slice(&rest)?,
            },
            1 => Self::Register  {
                metadata: RegisterMeta::try_from_slice(&rest)?,
            },
            2 => Self::CreateUserAccount,
            3 => Self::NewFollower {
                metadata: UserMeta::try_from_slice(&rest)?,
            },
            4 => Self::SetError {
                metadata: ErrorMeta::try_from_slice(&rest)?,
            },
            5 => Self::CheckFollower,
            6 => Self::CheckHashTag {
                metadata: HashTagMeta::try_from_slice(&rest)?,
            },
            7 => Self::SendTokens {
                metadata: HashTagRewardMeta::try_from_slice(&rest)?,
            },
            8 => Self::CheckRetweet {
                metadata: HashTagMeta::try_from_slice(&rest)?,
            },
            9 => Self::SetUserID {
                metadata: UserMeta::try_from_slice(&rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }
}