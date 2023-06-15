use borsh::{BorshDeserialize, BorshSerialize};



#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ScoreMeta {
    // the total number of teams registered
    pub high_score : u64,
}

pub fn get_score_meta_size() -> usize {
    let encoded = ScoreMeta {high_score: 0}
        .try_to_vec().unwrap();

    encoded.len()
}