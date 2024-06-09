pub mod state;

use crate::state::TokenInstruction;
use std::env;
use std::str::{from_utf8, FromStr};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Keypair,
    signer::keypair::read_keypair_file,
    signer::Signer,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;

const URL: &str = "https://api.devnet.solana.com";
//const URL: &str = "http://127.0.0.1:8899";

const PROGRAM_PUBKEY: &str = "DQ8rUMXqfvD6HYVt15ci9xLETZZuz49z5Q1i8tXNWSdq";

fn main() {
    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let function = &args[2];

    if function == "init" {
        if let Err(err) = init(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
}

pub fn init(key_file: &String) -> std::result::Result<(), state::Error> {
    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();

    let (pda_account, _bump_seed) = Pubkey::find_program_address(&[b"pda"], &program);

    let instruction = Instruction::new_with_borsh(
        program,
        &TokenInstruction::Init,
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(pda_account, false),
            AccountMeta::new(solana_sdk::system_program::id(), false),
        ],
    );

    let signers = [&wallet];
    let instructions = vec![instruction];
    let recent_hash = connection.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );

    let signature = connection.send_and_confirm_transaction_with_spinner_and_config(
        &txn,
        CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        },
        RpcSendTransactionConfig {
            skip_preflight: true,
            preflight_commitment: None,
            encoding: None,
            max_retries: None,
            min_context_slot: None,
        },
    )?;
    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);

    Ok(())
}
