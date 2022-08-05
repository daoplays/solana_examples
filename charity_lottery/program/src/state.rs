use borsh::{BorshDeserialize, BorshSerialize};
use enum_map::{Enum};
use solana_program::{
    pubkey::Pubkey,
};

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
    pub index : usize
}

pub struct State {

    pub select_winners: bool,
    pub prev_selection_time: i64,

    pub n_bidders: u32,
    pub bid_index : usize,
    pub total_bid_amount : u64,
    pub bid_keys : [Pubkey; 1024],
    pub bid_amounts: [u64; 1024],

    pub n_winners : u8,
    pub winners: [Pubkey; 128],

    pub charity_data : CharityData
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct BidValues {
    pub bid_amounts: [u64; 256],

}

// we can't unpack  the whole state in one go due to stack limits on chain.
// we create an enum to make it easier to access elements  within the state
pub enum StateEnum {

    SelectWinners, 
    PrevSelectionTime,

    NBidders,
    BidIndex,
    TotalBidAmount,
    BidKeys{
        index: usize
    },
    BidAmounts{
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

        // select_winners is a bool, so 1 byte
        StateEnum::SelectWinners => {(0, 1)},
        // the unix timestamp that winners were last selected, 8 bytes
        StateEnum::PrevSelectionTime => {(1, 9)}
        // the number of bidders, 4 bytes
        StateEnum::NBidders => {(9, 13)}
        // the index into the bidders array of the most recent bidder, 8 bytes
        StateEnum::BidIndex => {(13, 21)},
        // the total amount bid currently in the ladder, 8 bytes
        StateEnum::TotalBidAmount => {(21, 29)},
        // the list of bidder pubkeys, each is 32 bytes
        StateEnum::BidKeys{index} => {(29 + index * 32, 29 + (index + 1) * 32)},
        // the list of corresponding bid amounts, each is 8 bytes
        StateEnum::BidAmounts{index} => {(32797 + index * 8, 32797 + (index + 1) * 8)},

        // the number of winners selected, 1 byte
        StateEnum::NWinners => {(40989, 40990)},
        // pubkeys of the selected winners, each is 32 bytes
        StateEnum::Winners{index} => {(40990 + index * 32, 40990 + (index + 1) * 32)},
        
        // the Charity data is 80 bytes
        StateEnum::CharityData => {(45086, 45166)}
    }
}

// helper function to return the size of the State so we can check the lamports required to be rent-exempt
pub fn get_state_size() -> usize {
    45166
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