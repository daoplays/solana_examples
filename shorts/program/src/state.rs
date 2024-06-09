use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Default, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum AccountType {
    #[default]
    Launch,
    Program,
    User,
    Join,
    MMUserData,
    MMLaunchData,
    AMM,
    TimeSeries,
    CollectionLaunch,
    NFTAssignment,
    NFTLookup,
}
