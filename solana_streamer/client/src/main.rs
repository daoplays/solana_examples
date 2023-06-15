pub mod state;

use std::env;
use std::str::FromStr;
use crate::state::{Result, Choice, ChoiceData, ChoiceInstruction};

use solana_client::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey};
use solana_sdk::{
    signer::Signer,
    instruction::{Instruction},
    transaction::Transaction, signer::keypair::read_keypair_file,
};
use solana_transaction_status::UiTransactionEncoding;


fn get_choice_from_int(index: u8) -> Choice {
    if index == 0 {
        return Choice::A;
    } 
    else  if index == 1 {
        return Choice::B;
    }
    else  if index == 2 {
        return Choice::C;
    } 
    else {
        return Choice::D;
    }
}

const URL: &str = "https://api.devnet.solana.com";


fn main() {

    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let choice_string = &args[2];
    let amount_string = &args[3];

    let choice_index : u8 = choice_string.parse().unwrap();
    let amount : u64 = amount_string.parse().unwrap();
    let choice = get_choice_from_int(choice_index);

    if let Err(err) = make_choice(key_file, choice, amount) {
        eprintln!("{:?}", err);
        std::process::exit(1);
    }
    
    

}

fn make_choice(key_file: &String, choice: Choice, amount : u64) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str("H73oSXtdJfuBz8JWwdqyG92D3txMqxPEhAhT23T8eHf5").unwrap();
  
    let choice_data =  ChoiceData{choice: choice, bid_amount: amount};


    let make_choice_idx = Instruction::new_with_borsh(
        program,
        &ChoiceInstruction::MakeChoice{choice_data : choice_data},
        vec![
        ],
    );

    // (7) Build transaction wrapping the create account instruction signed by both accounts
    let signers = [&wallet];
    let instructions = vec![make_choice_idx];
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
