use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CreateMeta {
    pub team_name : String
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

pub fn get_score_meta_size() -> usize {
    let encoded = ScoreMeta {num_teams: 0, top_ten_teams : [0; 10], top_ten_scores : [0; 10]}
        .try_to_vec().unwrap();

    encoded.len()
}

/// Determines and reports the size of greeting data.
pub fn get_team_account_meta_size() -> usize {
    let encoded = TeamAccountMeta {team_account : solana_program::system_program::id() }
        .try_to_vec().unwrap();

    encoded.len()
}

/// Determines and reports the size of greeting data.
pub fn get_team_meta_size() -> usize {
    let encoded = TeamMeta {team_name : [0; 256], name_len : 0, mint_address : solana_program::system_program::id(), score : 0, index : 0}
        .try_to_vec().unwrap();

    encoded.len()
}