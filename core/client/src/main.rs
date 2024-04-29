pub mod state;

use crate::state::TokenInstruction;
use std::env;
use std::str::{from_utf8, FromStr};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Keypair,
    signer::keypair::read_keypair_file,
    signer::Signer,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;

const URL: &str = "https://api.devnet.solana.com";
const PROGRAM_PUBKEY: &str = "326p2wwfCBU36E52VShvsuh8WHkLenmbnEbxbKf3ja6z";

fn main() {
    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let function = &args[2];

    if function == "collection" {
        if let Err(err) = collection(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "token" {
        if let Err(err) = token(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
}

pub fn collection(key_file: &String) -> std::result::Result<(), state::Error> {
    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();

    let core = Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap();

    let asset = Keypair::new();

    let instruction = Instruction::new_with_borsh(
        program,
        &TokenInstruction::CreateCollection,
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(asset.pubkey(), true),
            AccountMeta::new(core, false),
            AccountMeta::new(solana_sdk::system_program::id(), false),
        ],
    );

    let signers = [&wallet, &asset];
    let instructions = vec![instruction];
    let recent_hash = connection.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );

    let signature = connection.send_and_confirm_transaction(&txn)?;
    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);

    Ok(())
}

pub fn token(key_file: &String) -> std::result::Result<(), state::Error> {
    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();

    let core = Pubkey::from_str("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d").unwrap();

    let collection = Pubkey::from_str("HQwfFQ7fnMcg4d8UVX2Q6QGkeKnMWqzR3Va5afzpugsV").unwrap();
    let asset = Keypair::new();

    let instruction = Instruction::new_with_borsh(
        program,
        &TokenInstruction::CreateToken,
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(asset.pubkey(), true),
            AccountMeta::new(collection, false),
            AccountMeta::new(core, false),
            AccountMeta::new(solana_sdk::system_program::id(), false),
        ],
    );

    let signers = [&wallet, &asset];
    let instructions = vec![instruction];
    let recent_hash = connection.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );

    let signature = connection.send_and_confirm_transaction(&txn)?;
    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);

    Ok(())
}
