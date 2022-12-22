pub mod state;

use std::env;
use std::str::{FromStr, from_utf8};
use crate::state::{Result, IceCreamInstruction, CreateMeta, ScoreMeta, TeamMeta};

use solana_client::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey};
use solana_sdk::{
    signature::Keypair, signer::Signer,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction, signer::keypair::read_keypair_file
};
use borsh::{BorshDeserialize};
use solana_transaction_status::UiTransactionEncoding;
use spl_associated_token_account::{get_associated_token_address};


const URL: &str = "https://api.devnet.solana.com";
const PROGRAM_PUBKEY:  &str = "4gvkmbyVPgiorM6uiQnqdPpSSidwLcwctbTfrt9bCxsn";

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

    if function == "create" {
        let team_name = &args[3];
        if let Err(err) = create(key_file, team_name) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "eat" {
        let team_name = &args[3];
        if let Err(err) = eat(key_file, team_name) {
          eprintln!("{:?}", err);
          std::process::exit(1);
      }
  }
}

pub fn init(
    key_file: &String
) -> Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();

    let (data_key, _data_seed) = Pubkey::find_program_address(&[b"data_account"], &program);
   
    let instruction = Instruction::new_with_borsh(
        program,
        &IceCreamInstruction::InitProgram,
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(data_key, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false)

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


pub fn create(
    key_file: &String,
    team_name: &String
) -> Result<()> {

    //let team_name = "the spoons".to_string();
    let meta_data =  CreateMeta{team_name : team_name.to_string()};

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();
    let mint_address = Keypair::new(); 

    let my_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &mint_address.pubkey()
    );

    let (data_key, _data_seed) = Pubkey::find_program_address(&[b"data_account"], &program);
    let (team_key, _team_seed) = Pubkey::find_program_address(&[meta_data.team_name.as_bytes()], &program);
   
    println!("{:?}", mint_address.pubkey().to_string());
    println!("{:?}", solana_sdk::sysvar::rent::ID);

    

    let instruction = Instruction::new_with_borsh(
        program,
        &IceCreamInstruction::CreateTeam {metadata : meta_data},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(mint_address.pubkey(), true),
            AccountMeta::new(my_token_address, false),

            AccountMeta::new(data_key, false),
            AccountMeta::new(team_key, false),

            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new(solana_sdk::system_program::id(), false),
            AccountMeta::new( solana_sdk::sysvar::rent::ID, false)
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


    let team_account = connection.get_account_data(&team_key)?;
    let current_state = TeamMeta::try_from_slice(&team_account[..]).unwrap();

    let name_bytes = current_state.team_name;

    let name_string = from_utf8(&name_bytes).unwrap();
    
    println!("team name: {} index {}", name_string, current_state.index);


    let (team_account_key, _team_seed) = Pubkey::find_program_address(&[&current_state.index.to_le_bytes()], &program);

    let meta_data_2 =  CreateMeta{team_name : team_name.to_string()};

    let create_idx_2 = Instruction::new_with_borsh(
        program,
        &IceCreamInstruction::CreateTeamLookup {metadata : meta_data_2},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
           
            AccountMeta::new(team_account_key, false),
            AccountMeta::new(team_key, false),

            
            AccountMeta::new(solana_sdk::system_program::id(), false)
        ],
    );

    let signers_2 = [&wallet];
    let instructions_2 = vec![create_idx_2];
    let recent_hash_2 = connection.get_latest_blockhash()?;

    let txn_2 = Transaction::new_signed_with_payer(
        &instructions_2,
        Some(&wallet.pubkey()),
        &signers_2,
        recent_hash_2,
    );

    let signature_2 = connection.send_and_confirm_transaction(&txn_2)?;
    println!("signature: {}", signature_2);
    let response_2 = connection.get_transaction(&signature_2, UiTransactionEncoding::Json)?;
    println!("result: {:#?}", response_2);


    Ok(())
}


pub fn eat(
    key_file: &String,
    team_name: &String
) -> Result<()> {

    let meta_data =  CreateMeta{team_name : team_name.to_string()};

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();

    let (data_key, _data_seed) = Pubkey::find_program_address(&[b"data_account"], &program);
    let (team_key, _team_seed) = Pubkey::find_program_address(&[meta_data.team_name.as_bytes()], &program);
   
    let mut team_account = connection.get_account_data(&team_key)?;
    let mut current_state = TeamMeta::try_from_slice(&team_account[..]).unwrap();

    //println!("data: {:?}", team_account);
    //println!("state: {:?}", current_state);

    let my_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &current_state.mint_address
    );

    println!("accounts:");
    println!("mint: {:?}", current_state.mint_address.to_string());
    println!("user's token address: {:?}", my_token_address.to_string());
    println!("program data key: {:?}", data_key.to_string());
    println!("team data key: {:?}", team_key.to_string());

    

    let instruction = Instruction::new_with_borsh(
        program,
        &IceCreamInstruction::Eat {metadata : meta_data},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),

            AccountMeta::new_readonly(current_state.mint_address, false),
            AccountMeta::new_readonly(my_token_address, false),

            AccountMeta::new(data_key, false),
            AccountMeta::new(team_key, false),

            AccountMeta::new_readonly(spl_associated_token_account::id(), false)
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


    team_account = connection.get_account_data(&team_key)?;

    println!("data: {:?}", team_account);
    current_state = TeamMeta::try_from_slice(&team_account[..]).unwrap();

    let name_bytes = current_state.team_name;

    let name_string = from_utf8(&name_bytes).unwrap();
    
    println!("team name: {} score {}", name_string, current_state.score);

    let scores = connection.get_account_data(&data_key)?;
    let current_scores = ScoreMeta::try_from_slice(&scores[..]).unwrap();

    println!("{:?}", current_scores.top_ten_scores);
    println!("{:?}", current_scores.top_ten_teams);


    Ok(())
}