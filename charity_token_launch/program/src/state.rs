use borsh::{BorshDeserialize, BorshSerialize};
use enum_map::{Enum};

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

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct JoinMeta {
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
    pub amount : u64,
    // the amount of supporter tokens to be send to the program
    pub supporter_amount : u64
}

// on chain data that saves summary stats of the token launch
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TokenLaunchData {
    // the total donated to each charity
    pub charity_totals : [u64 ; 7],
    // the total donated overall
    pub donated_total : u64,
    // the total paid overall
    pub paid_total : u64,
    // the number of participating accounts
    pub n_donations : u64
}

// helper function to return the size of the TokenLaunchData so we can check the lamports required to be rent-exempt
pub fn get_state_size() -> usize {
    let encoded = TokenLaunchData {charity_totals: [0; 7], donated_total : 0, paid_total : 0, n_donations : 0}
        .try_to_vec().unwrap();

    encoded.len()
}