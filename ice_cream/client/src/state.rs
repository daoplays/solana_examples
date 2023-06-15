use borsh::{BorshDeserialize, BorshSerialize};
use thiserror::Error;
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
pub struct CreateMeta {
    pub team_name : String
}



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

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ScoreMeta {
    // the total number of teams registered
    pub num_teams : u64,
    // the indices of the top ten teams
    pub top_ten_teams : [u64; 10],
    // the scores of the top ten teams
    pub top_ten_scores : [u64; 10],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TeamAccountMeta {
    // the total number of teams registered
    pub team_account : Pubkey

}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TeamMeta {
    pub team_name : [u8 ; 256],
    pub name_len : u64,
    // the mint address of this team
    pub mint_address : Pubkey,
    // the teams score
    pub score : u64,
    // the teams index
    pub index : u64
}
