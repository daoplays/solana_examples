use borsh::{BorshDeserialize, BorshSerialize};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum Error {

    #[error("solana client error: ({0})")]
    ClientError(#[from] solana_client::client_error::ClientError),

}

pub type Result<T> = std::result::Result<T, Error>;


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum ChoiceInstruction {

    // MakeChoice expects only one account, the user of the program which should be signed
    MakeChoice {
        choice_data: ChoiceData
    }
}

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