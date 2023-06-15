use borsh::{BorshDeserialize, BorshSerialize};
use enum_map::{Enum};
use solana_program::{
    pubkey::Pubkey,
};

// the max number of winners we can select in one go
pub const MAX_BIDDERS : usize = 1024;
pub const MAX_WINNERS : usize = 4;
pub const TOKENS_WON : u64 = 100;

pub const BID_BLOCK : usize = 64;
pub const N_BID_BLOCKS : usize = 16;




pub struct SeedStruct {
    pub seed_prices : [u64;  9]
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
    TheLifeYouCanSave

}

// on chain data that saves summary stats of the token launch
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
pub struct BidderData {
    pub index : u16
}

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

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct BidValues {
    pub bid_amounts: [u64; BID_BLOCK],

}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct BidTimes {
    pub bid_times: [i64; BID_BLOCK],

}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct WinnersKeys {
    pub keys: [Pubkey; MAX_WINNERS],

}

// we can't unpack  the whole state in one go due to stack limits on chain.
// we create an enum to make it easier to access elements  within the state
pub enum StateEnum {

    PrevSelectionTime,

    NBidders,
    TotalBidAmount,

    BidKeys{
        index: usize
    },
    BidAmounts{
        index: usize
    },
    BidTimes{
        index: usize
    },

    NWinners,
    Winners{
        index: usize
    },

    CharityData 

    
}

pub fn get_state_index(element: StateEnum) -> (usize, usize) {

    match element {

        // the unix timestamp that winners were last selected, 8 bytes
        StateEnum::PrevSelectionTime => {(0, 8)}
    
        // the number of bidders, 2 bytes
        StateEnum::NBidders => {(8, 10)}
        // the total amount bid currently in the ladder, 8 bytes
        StateEnum::TotalBidAmount => {(10, 18)},

        // the list of bidder pubkeys, each is 32 bytes
        StateEnum::BidKeys{index} => {(18 + index * 32, 18 + (index + 1) * 32)},
        // the list of corresponding bid amounts, each is 8 bytes
        StateEnum::BidAmounts{index} => {(32786 + index * 8, 32786 + (index + 1) * 8)},
        // the list of corresponding bid amounts, each is 8 bytes
        StateEnum::BidTimes{index} => {(40978 + index * 8, 40978 + (index + 1) * 8)},

        // the number of winners selected, 1 byte
        StateEnum::NWinners => {(49170, 49171)},
        // pubkeys of the selected winners, each is 32 bytes
        StateEnum::Winners{index} => {(49171 + index * 32, 49171 + (index + 1) * 32)},
        
        // the Charity data is 80 bytes
        StateEnum::CharityData => {(49299, 49379)}
    }
}

// helper function to return the size of the State so we can check the lamports required to be rent-exempt
pub fn get_state_size() -> usize {
    49379
}

/// Determines and reports the size of greeting data.
pub fn get_charity_size() -> usize {
    let encoded = CharityData {charity_totals: [0; 7], donated_total : 0, paid_total : 0, n_donations : 0}
        .try_to_vec().unwrap();

    encoded.len()
}

/// Determines and reports the size of greeting data.
pub fn get_bid_status_size() -> usize {
    let encoded = BidderData {index: 0}
        .try_to_vec().unwrap();

    encoded.len()
}