
pub mod utils;
pub mod state;

use crate::state::{Result, TwitterInstruction, RegisterMeta, UserMeta, TokenMeta, IDMap, UserData, HashTagMeta};

use std::env;
use std::str::FromStr;
use solana_client::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey, rent, native_token::LAMPORTS_PER_SOL, system_program};
use solana_sdk::{
    signer::Signer,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction, signer::keypair::read_keypair_file, hash
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_transaction_status::UiTransactionEncoding;
use spl_associated_token_account::{get_associated_token_address};
use enum_map::{enum_map, Enum};

// some globals
const PROGRAM_KEY : &str = "3NxKJZcBRyw7eAVg3GwA22YP2DPgqSWpiS3SG1jbnUbZ";
const MINT_KEY : &str =  "ESxUiMdmrZzJBk1JryJyVpD2ok9cheTV43H1HXEy8n5x";
const DAOPLAYS : &str =  "FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD";

const SOLANA_DEV: &str = "https://api.devnet.solana.com";

const URL: &str = SOLANA_DEV;

fn main() {
    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let function = &args[2];

    if function == "register" {

        let tweet_id = &args[3];
        let tweet_64 : u64 = tweet_id.parse().unwrap();

        if let Err(err) = register(key_file, tweet_64) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "init_program" {

        let amount_arg = &args[3];
        let amount: u64 = amount_arg.parse().unwrap();

        if let Err(err) = init_program(key_file, amount) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "create_account" {

        let user_id_string = &args[3];
        let user_id: u64 = user_id_string.parse().unwrap();

        if let Err(err) = create_account(key_file, user_id) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "monitor_data" {

        if let Err(err) = monitor_data(key_file) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "check_new_follower" {

        let user_id_string = &args[3];
        let user_id: u64 = user_id_string.parse().unwrap();

        if let Err(err) = check_new_follower(key_file, user_id) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "check_hashtag" {

        let tweet_id_string = &args[3];
        let tweet_id: u64 = tweet_id_string.parse().unwrap();

        let hashtag = &args[4];

        if let Err(err) = check_hashtag(key_file, tweet_id, hashtag.to_string()) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

    if function == "check_retweet" {

        let tweet_id_string = &args[3];
        let tweet_id: u64 = tweet_id_string.parse().unwrap();

        let hashtag = &args[4];

        if let Err(err) = check_retweet(key_file, tweet_id, hashtag.to_string()) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }

}

fn init_program(key_file: &String, amount : u64) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_KEY).unwrap();
    let daoplays  = Pubkey::from_str(DAOPLAYS).unwrap();

    let (expected_program_pda, program_bump_seed) = Pubkey::find_program_address(&[b"token_account"], &program);

    let supporter_mint_address = Pubkey::from_str(MINT_KEY).unwrap();
    let my_supporter_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &supporter_mint_address
    );

    let program_supporter_token_address = get_associated_token_address(
        &expected_program_pda, 
        &supporter_mint_address
    );



    let meta_data =  TokenMeta{supporter_amount : amount};
 
    println!("program pda {} {}", expected_program_pda, program_bump_seed);



    let instruction = Instruction::new_with_borsh(
        program,
        &TwitterInstruction::InitProgram { metadata: meta_data},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(expected_program_pda, false),

            AccountMeta::new(my_supporter_token_address, false),
            AccountMeta::new(program_supporter_token_address, false),

            AccountMeta::new_readonly(supporter_mint_address, false),

            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new(solana_sdk::system_program::id(), false)
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


    Ok(println!("Success!"))
}



fn register(key_file: &String, tweet_id : u64) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_KEY).unwrap();
    let daoplays  = Pubkey::from_str(DAOPLAYS).unwrap();

    let supporter_mint_address = Pubkey::from_str(MINT_KEY).unwrap();
    let my_supporter_token_address = get_associated_token_address(
        &wallet.pubkey(), 
        &supporter_mint_address
    );

    let (expected_user_pda, _user_bump_seed) = Pubkey::find_program_address(&[&wallet.pubkey().to_bytes()], &program);

    println!("Registering with tweet {}", tweet_id);
    let meta_data =  RegisterMeta{tweet_id : tweet_id};
 
    println!("user id map {}", expected_user_pda);
    let instruction = Instruction::new_with_borsh(
        program,
        &TwitterInstruction::Register { metadata: meta_data},
        vec![
            AccountMeta::new_readonly(wallet.pubkey(), true),
            AccountMeta::new(my_supporter_token_address, false),
            AccountMeta::new(expected_user_pda, false),

            AccountMeta::new(daoplays, false),
            AccountMeta::new_readonly(supporter_mint_address, false),

            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new(solana_sdk::system_program::id(), false)
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


    Ok(println!("Success!"))
}

fn create_account(key_file: &String, user_id : u64) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_KEY).unwrap();
    let user_account = Pubkey::from_str("7LtYL85tZPpYweZMqeHzX6DAaGsrY61DEtnwiPyJaVCD").unwrap();

    let (user_data_account, _user_bump_seed) = Pubkey::find_program_address(&[&user_id.to_le_bytes()], &program);
 
    let (user_id_map, _user_id_map_bump_seed) = Pubkey::find_program_address(&[&user_account.to_bytes()], &program);

    println!("Registering with user id {}", user_id);
    let meta_data =  UserMeta{user_id : user_id};

    println!("user data account {}", user_data_account);
    let instruction = Instruction::new_with_borsh(
        program,
        &TwitterInstruction::CreateUserAccount { metadata: meta_data},
        vec![
            AccountMeta::new(wallet.pubkey(), true),

            AccountMeta::new_readonly(user_account, false),
            AccountMeta::new(user_data_account, false),
            AccountMeta::new(user_id_map, false),


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


    Ok(println!("Success!"))
}


fn check_new_follower(key_file: &String, user_id : u64) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_KEY).unwrap();
    let user_account = Pubkey::from_str("7LtYL85tZPpYweZMqeHzX6DAaGsrY61DEtnwiPyJaVCD").unwrap();

    let (user_data_account, _user_bump_seed) = Pubkey::find_program_address(&[&user_id.to_le_bytes()], &program);
 
    let (user_id_map, _user_id_map_bump_seed) = Pubkey::find_program_address(&[&user_account.to_bytes()], &program);

    let supporter_mint_address = Pubkey::from_str(MINT_KEY).unwrap();
    let user_supporter_token_address = get_associated_token_address(
        &user_account, 
        &supporter_mint_address
    );

    let (expected_program_pda, program_bump_seed) = Pubkey::find_program_address(&[b"token_account"], &program);

    println!("byes {:?}", [b"token_account"]);


    let program_supporter_token_address = get_associated_token_address(
        &expected_program_pda, 
        &supporter_mint_address
    );


    println!("Registering with user id {}", user_id);
    let meta_data =  UserMeta{user_id : user_id};

    println!("user data account {}", user_data_account);


    let instruction = Instruction::new_with_borsh(
        program,
        &TwitterInstruction::NewFollower { metadata: meta_data},
        vec![
            AccountMeta::new(wallet.pubkey(), true),

            AccountMeta::new_readonly(user_account, false),
            AccountMeta::new(user_data_account, false),
            AccountMeta::new(user_supporter_token_address, false),

            AccountMeta::new(expected_program_pda, false),
            AccountMeta::new(program_supporter_token_address, false),


            AccountMeta::new_readonly(supporter_mint_address, false),

            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new(solana_sdk::system_program::id(), false)
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


    Ok(println!("Success!"))
}


pub fn monitor_data(key_file: &String) -> Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();


    // (2) Create a new Keypair for the new account
    let program = Pubkey::from_str(PROGRAM_KEY).unwrap();

    let (expected_user_pda, _user_bump_seed) = Pubkey::find_program_address(&[&wallet.pubkey().to_bytes()], &program);

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let response = connection.get_account_data(&expected_user_pda)?;
    println!("data in account: {}", expected_user_pda);
    //println!("{:#?}", response);

    let current_state = IDMap::try_from_slice(&response[..]).unwrap();

    println!("data: twitter_id {} error_code: {}", current_state.twitter_id, current_state.error_code);

    let (user_data_account, _data_bump_seed) = Pubkey::find_program_address(&[&current_state.twitter_id.to_le_bytes()], &program);

    let data_response = connection.get_account_data(&user_data_account)?;
    println!("data in account: {} ", user_data_account);
    //println!("{:#?}", data_response);

    let data_state = UserData::try_from_slice(&data_response[..]).unwrap();

    println!("data: account_key {} last_time: {} follow: {}", data_state.account_key, data_state.last_time, data_state.follow);


    Ok(())
}



fn check_hashtag(key_file: &String, tweet_id : u64, hashtag : String) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_KEY).unwrap();
 
    let (user_id_map, _user_id_map_bump_seed) = Pubkey::find_program_address(&[&wallet.pubkey().to_bytes()], &program);

    let response = connection.get_account_data(&user_id_map)?;
    println!("data in account: {}", user_id_map);

    let current_state = IDMap::try_from_slice(&response[..]).unwrap();

    let twitter_id = current_state.twitter_id;

    let (user_hashtag_key, _user_hashtag_bump_seed) = Pubkey::find_program_address(&[hashtag.as_bytes(), &tweet_id.to_le_bytes(), &twitter_id.to_le_bytes()], &program);

    println!("key1!: {:?}", hashtag.as_bytes());
    println!("key2!: {:?}", tweet_id.to_le_bytes());
    println!("key3!: {:?}", twitter_id.to_le_bytes());
    println!("data key: {}", user_hashtag_key.to_string());

    return Ok(());

    let meta_data =  HashTagMeta{tweet_id : tweet_id, hashtag : hashtag};

    let instruction = Instruction::new_with_borsh(
        program,
        &TwitterInstruction::CheckHashTag { metadata: meta_data},
        vec![
            AccountMeta::new(wallet.pubkey(), true),
            AccountMeta::new_readonly(user_id_map, false),
            AccountMeta::new(user_hashtag_key, false),

            AccountMeta::new(Pubkey::from_str(DAOPLAYS).unwrap(), false),

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


    Ok(println!("Success!"))
}

fn check_retweet(key_file: &String, tweet_id : u64, hashtag : String) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_KEY).unwrap();
 
    let (user_id_map, _user_id_map_bump_seed) = Pubkey::find_program_address(&[&wallet.pubkey().to_bytes()], &program);

    let response = connection.get_account_data(&user_id_map)?;
    println!("data in account: {}", user_id_map);

    let current_state = IDMap::try_from_slice(&response[..]).unwrap();

    let twitter_id = current_state.twitter_id;

    let (user_hashtag_key, _user_hashtag_bump_seed) = Pubkey::find_program_address(&[hashtag.as_bytes(), &tweet_id.to_le_bytes(), &twitter_id.to_le_bytes()], &program);

    println!("key1!: {:?}", hashtag.as_bytes());
    println!("key2!: {:?}", tweet_id.to_le_bytes());
    println!("key3!: {:?}", twitter_id.to_le_bytes());
    println!("data key: {}", user_hashtag_key.to_string());



    let meta_data =  HashTagMeta{tweet_id : tweet_id, hashtag : hashtag};

    let instruction = Instruction::new_with_borsh(
        program,
        &TwitterInstruction::CheckRetweet { metadata: meta_data},
        vec![
            AccountMeta::new(wallet.pubkey(), true),
            AccountMeta::new_readonly(user_id_map, false),
            AccountMeta::new(user_hashtag_key, false),

            AccountMeta::new(Pubkey::from_str(DAOPLAYS).unwrap(), false),

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


    Ok(println!("Success!"))
}
