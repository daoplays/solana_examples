use thiserror::Error;
use borsh::{BorshDeserialize, BorshSerialize};
use enum_map::{enum_map, Enum};
use solana_program::{pubkey::Pubkey};

pub const MAX_BIDDERS : usize = 1024;
pub const MAX_WINNERS : usize = 4;
pub const TOKENS_WON : u64 = 1;
pub const BID_BLOCK : usize = 128;
pub const KEY_BLOCK : usize = 16;
pub const N_BID_BLOCKS : usize = 8;
pub const N_KEY_BLOCKS : usize = 64;



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
pub struct InitMeta {
    // the amount of DPTTs to be sent to the program
    pub amount : u64
}

// enum that lists the supported charities for this token launch
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Enum, Copy)]
pub enum Charity {

    UkraineERF,
    WaterOrg,
    OneTreePlanted,
    EvidenceAction,
    GirlsWhoCode,
    OutrightActionInt,
    TheLifeYouCanSave,
    InvalidCharity

}

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
pub enum AuctionInstruction {

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

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CharityData {
    // the total donated to each charity
    pub charity_totals : [u64 ; 7],
    // the total donated overall
    pub donated_total : u64,
    // the total paid overall
    pub paid_total : u64,
    // the number of participating accounts
    pub n_donations : u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct State {

    // this is the last time we actually chose winners, and decides how soon in the future will we choose again
    pub prev_choose_winners_time: i64,

    // the number of active bids in the system up to MAX_BIDDERS
    pub n_bidders: u16,
    // the sum of all the current bids
    pub total_bid_amount : u64,

    // for each bid we track the key, amount and time
    pub bid_keys : [Pubkey; MAX_BIDDERS],
    pub bid_amounts: [u64; MAX_BIDDERS],
    pub bid_times: [i64; MAX_BIDDERS],

    // the number of winners to be chosen, up to MAX_WINNERS
    pub n_winners : u8,
    pub winners: [Pubkey; MAX_WINNERS],

    // summary of the charity stats for the auction
    pub charity_data : CharityData
}