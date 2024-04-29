use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Default)]
pub struct MintData {
    pub count: u64,
}

pub fn get_mint_data_size() -> usize {
    8
}
