use borsh::{BorshDeserialize, BorshSerialize};


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum Choice {
    A,
    B,
    C,
    D
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ChoiceData {
    pub choice : Choice,
    pub bid_amount : u64
}
