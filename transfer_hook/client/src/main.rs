pub mod state;

use crate::state::{Result, TokenInstruction};
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

    let program = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();
    let mint_address = Keypair::new();

    let my_token_address = get_associated_token_address_with_program_id(
        &wallet.pubkey(),
        &mint_address.pubkey(),
        &spl_token_2022::ID,
    );

    let transfer = state::Extensions::TransferFee as u8;
    let delegate = state::Extensions::PermanentDelegate as u8;
    let interest = state::Extensions::InterestBearing as u8;
    let transferable = state::Extensions::NonTransferable as u8;
    let default = state::Extensions::DefaultState as u8;

    let included_extensions: u8 = transfer | transferable;

    let include_transfer: u8 = included_extensions & transfer;
    let include_delegate: u8 = included_extensions & delegate;
    let include_interest: u8 = included_extensions & interest;
    let include_transferable: u8 = included_extensions & transferable;
    let include_default_state: u8 = included_extensions & default;

    println!(
        "values : {} {} {} {} {} {}",
        included_extensions,
        include_transfer > 0,
        include_delegate > 0,
        include_interest > 0,
        include_transferable > 0,
        include_default_state > 0
    );

    let meta_data = state::CreateMeta {
        extensions: included_extensions,
    };
    let instruction = Instruction::new_with_borsh(
        program,
        &TokenInstruction::CreateToken {
            metadata: meta_data,
        },
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(mint_address.pubkey(), true),
            AccountMeta::new(my_token_address, false),
            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new(solana_sdk::system_program::id(), false),
        ],
    );

    let signers = [&wallet, &mint_address];
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
