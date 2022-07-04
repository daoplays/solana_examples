pub mod state;

use std::env;
use std::str::FromStr;
use crate::state::{Result};

use solana_client::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey};
use solana_sdk::{
    signature::Keypair, signer::Signer,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction, signer::keypair::read_keypair_file,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_transaction_status::UiTransactionEncoding;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum RNGInstruction {

    GenerateSeed
}
const URL: &str = "https://api.devnet.solana.com";


fn main() {

    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let function = &args[2];

    if function == "generate_seed" {
        
        if let Err(err) = generate_seed(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
    

}

fn generate_seed(key_file: &String) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str("Hqw9GzaxEg1efH8BciNN5D32A5fMAfBdDM3qudRdb9o5").unwrap();
    let pyth_key = Pubkey::from_str("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J").unwrap();

    
    let gen_seed_idx = Instruction::new_with_borsh(
        program,
        &RNGInstruction::GenerateSeed,
        vec![
            AccountMeta::new_readonly(pyth_key, false)
        ],
    );

    // (7) Build transaction wrapping the create account instruction signed by both accounts
    let signers = [&wallet];
    let instructions = vec![gen_seed_idx];
    let recent_hash = connection.get_latest_blockhash()?;

    let txn = Transaction::new_signed_with_payer(
        &instructions,
        Some(&wallet.pubkey()),
        &signers,
        recent_hash,
    );

    // (8) Send transaction to the cluster and wait for confirmation
    let signature = connection.send_and_confirm_transaction(&txn)?;
    println!("signature: {}", signature);
    let response = connection.get_transaction(&signature, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response);


    Ok(println!("Success!"))
}
