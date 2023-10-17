pub mod state;

use crate::state::{Result, TransferHookInstruction};
use std::env;
use std::str::{from_utf8, FromStr};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_client::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey, sysvar::rent};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signature::Keypair,
    signer::keypair::read_keypair_file,
    signer::Signer,
    transaction::Transaction,
};
use solana_transaction_status::UiTransactionEncoding;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use spl_token_2022;

const URL: &str = "https://api.devnet.solana.com";
const PROGRAM_PUBKEY: &str = "8ZMLymiBfEWkZwaRebKhFXgUGbEdpnjij36i5PULFHSX";
const HOOK_PUBKEY: &str = "vyyNeAorB3Ce4nyfBK4dL7CMu9Jx2M9vK8zBGvFrpYd";

fn main() {
    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let function = &args[2];

    if function == "create" {
        if let Err(err) = create(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
    if function == "transfer" {
        if let Err(err) = transfer(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
}
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Payload {
    variant: u8,
    extension: u16,
}
pub fn create(key_file: &String) -> Result<()> {
    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let hook_program = Pubkey::from_str(HOOK_PUBKEY).unwrap();


    let mint_address = Pubkey::from_str("6PyymMZ3TXQSn9hW6fcwrpu3GQ7PrUtfevFNe3rvkX3T").unwrap();
    let (expected_validation_address, bump_seed) =
    state::get_extra_account_metas_address_and_bump_seed(&mint_address, &hook_program);

  
    let instruction = Instruction::new_with_borsh(
        hook_program,
        &TransferHookInstruction::InitializeExtraAccountMetas ,
        vec![
            AccountMeta::new(expected_validation_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new_readonly(wallet.pubkey(), true),

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

    let signature = connection.send_and_confirm_transaction(&txn)?;
    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);

    Ok(())
}


pub fn transfer(key_file: &String) -> Result<()> {
    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let hook_program = Pubkey::from_str(HOOK_PUBKEY).unwrap();


    let mint_address = Pubkey::from_str("CJpBCgjjJc1ES36Zahq4zRs85quLxtbiMRaFRVYQK2RZ").unwrap();
    let (expected_validation_address, bump_seed) =
    state::get_extra_account_metas_address_and_bump_seed(&mint_address, &hook_program);

  
    let instruction = Instruction::new_with_borsh(
        hook_program,
        &TransferHookInstruction::InitializeExtraAccountMetas ,
        vec![
            AccountMeta::new(expected_validation_address, false),
            AccountMeta::new(mint_address, false),
            AccountMeta::new_readonly(wallet.pubkey(), true),

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

    let signature = connection.send_and_confirm_transaction(&txn)?;
    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);

    Ok(())
}