use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use thiserror::Error;
use spl_discriminator::{ArrayDiscriminator, SplDiscriminate};

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read solana config file: ({0})")]
    ConfigReadError(std::io::Error),

    #[error("invalid config: ({0})")]
    InvalidConfig(String),

    #[error("serialization error: ({0})")]
    SerializationError(std::io::Error),

    #[error("solana client error: ({0})")]
    ClientError(#[from] solana_client::client_error::ClientError),

    #[error("error in public key derivation: ({0})")]
    KeyDerivationError(#[from] solana_sdk::pubkey::PubkeyError),
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CreateMeta {
    pub extensions: u8,
    pub transfer_fee_bp: u16,
    pub transfer_fee_max: u64,
    pub interest_rate: i16,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TokenInstruction {
    CreateToken { metadata: CreateMeta },
    Test
}

pub enum Extensions {
    None = 0,
    TransferFee = 1,
    PermanentDelegate = 2,
    InterestBearing = 4,
    NonTransferable = 8,
    DefaultState = 16,
    TransferHook = 32,
}



/// Namespace for all programs implementing transfer-hook
pub const NAMESPACE: &str = "spl-transfer-hook-interface";

/// Seed for the state
const EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"extra-account-metas";

/// Get the state address PDA
pub fn get_extra_account_metas_address(mint: &Pubkey, program_id: &Pubkey) -> Pubkey {
    get_extra_account_metas_address_and_bump_seed(mint, program_id).0
}

/// Function used by programs implementing the interface, when creating the PDA,
/// to also get the bump seed
pub fn get_extra_account_metas_address_and_bump_seed(
    mint: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&collect_extra_account_metas_seeds(mint), program_id)
}

/// Function used by programs implementing the interface, when creating the PDA,
/// to get all of the PDA seeds
pub fn collect_extra_account_metas_seeds(mint: &Pubkey) -> [&[u8]; 2] {
    [EXTRA_ACCOUNT_METAS_SEED, mint.as_ref()]
}

/// Function used by programs implementing the interface, when creating the PDA,
/// to sign for the PDA
pub fn collect_extra_account_metas_signer_seeds<'a>(
    mint: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [EXTRA_ACCOUNT_METAS_SEED, mint.as_ref(), bump_seed]
}

#[derive(SplDiscriminate)]
#[discriminator_hash_input("spl-transfer-hook-interface:execute")]
pub struct ExecuteInstruction;