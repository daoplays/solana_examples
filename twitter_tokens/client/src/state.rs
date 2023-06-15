use thiserror::Error;
use borsh::{BorshDeserialize, BorshSerialize};
use enum_map::{enum_map, Enum};
use solana_program::{pubkey::Pubkey};


#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read solana config file: ({0})")]
    ConfigReadError(std::io::Error),
   
    #[error("invalid config: ({0})")]
    InvalidConfig(String),

    #[error("serialization error: ({0})")]
    SerializationError(std::io::Error),

    #[error("solana client error: ({0})")]
    ClientError(#[from] solana_client::client_error::ClientError),

    #[error("error in public key derivation: ({0})")]
    KeyDerivationError(#[from] solana_sdk::pubkey::PubkeyError),
}

pub type Result<T> = std::result::Result<T, Error>;


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct RegisterMeta {
    // the string of the id that contains the users pubkey
    pub tweet_id : u64
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct UserMeta {
    // the string of the id that contains the users pubkey
    pub user_id : u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TokenMeta {
    // the amount of supporter tokens to be send to the program
    pub supporter_amount : u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ErrorMeta {
    // the amount of supporter tokens to be send to the program
    pub error_code : u8
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct HashTagMeta {
    // the string of the id that contains the users pubkey
    pub tweet_id : u64,
    pub hashtag : String
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
    CreateUserAccount {
        metadata : UserMeta
    },
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
    }
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct UserData {
    pub account_key : Pubkey,
    pub last_time : i64,
    pub follow : bool
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct IDMap {
    pub twitter_id : u64,
    pub error_code : u8
}

/// Determines and reports the size of user data.
pub fn get_user_data_size() -> usize {
    let encoded = UserData {account_key: solana_program::system_program::id(), last_time: 0, follow: false}
        .try_to_vec().unwrap();

    encoded.len()
}

pub fn get_id_map_size() -> usize {
    let encoded = IDMap {twitter_id: 0, error_code: 0}
        .try_to_vec().unwrap();

    encoded.len()
}