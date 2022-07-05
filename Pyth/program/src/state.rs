use borsh::{BorshDeserialize, BorshSerialize};

pub struct SeedStruct {
    pub seed_prices : [u64;  9]
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum SeedMethod {
    ShiftMurmur,
    SHA256Hash,
    None
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct SeedMeta {
    pub method : SeedMethod
}