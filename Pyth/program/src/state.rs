use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct State {
    pub random_numbers : [f64; 256]
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum RNGMethod {
    Xorshift,
    Hash,
    FastHash,
    None
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct RNGMeta {
    pub initial_seed : u64,
    pub method : RNGMethod
}

pub struct HashStruct {
    pub nonce : u64,
    pub initial_seed : u64
}

pub struct SeedStruct {
    pub seed_prices : [u64;  9]
}