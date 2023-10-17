pub mod state;

use crate::state::{TokenInstruction};
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
use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList, seeds::Seed};
use spl_type_length_value::state::TlvStateBorrowed;


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
    if function == "test" {
        if let Err(err) = test(key_file) {
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

pub fn create(key_file: &String) -> std::result::Result<(), state::Error> {
    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str(PROGRAM_PUBKEY).unwrap();
    let hook_program = Pubkey::from_str(HOOK_PUBKEY).unwrap();

    let mint_address = Keypair::new();

    let my_token_address = get_associated_token_address_with_program_id(
        &wallet.pubkey(),
        &mint_address.pubkey(),
        &spl_token_2022::ID,
    );

    let (mint_data_account, _old_bump_seed) = Pubkey::find_program_address(&[b"mint_data", &mint_address.pubkey().to_bytes()], &hook_program);


    let (expected_validation_address, bump_seed) =
    state::get_extra_account_metas_address_and_bump_seed(&mint_address.pubkey(), &hook_program);

    println!(" extra for mint {} {}", mint_address.pubkey().to_string(), expected_validation_address.to_string());

    let transfer = state::Extensions::TransferFee as u8;
    let delegate = state::Extensions::PermanentDelegate as u8;
    let interest = state::Extensions::InterestBearing as u8;
    let transferable = state::Extensions::NonTransferable as u8;
    let default = state::Extensions::DefaultState as u8;
    let transfer_hook = state::Extensions::TransferHook as u8;


    let included_extensions: u8 = transfer_hook;

    let include_transfer: u8 = included_extensions & transfer;
    let include_delegate: u8 = included_extensions & delegate;
    let include_interest: u8 = included_extensions & interest;
    let include_transferable: u8 = included_extensions & transferable;
    let include_default_state: u8 = included_extensions & default;
    let include_transfer_hook: u8 = included_extensions & transfer_hook;

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
        transfer_fee_bp: 0,
        transfer_fee_max: 0,
        interest_rate: 0,
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
            AccountMeta::new_readonly(hook_program, false),
            AccountMeta::new(expected_validation_address, false),

            AccountMeta::new_readonly(spl_token_2022::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new(solana_sdk::system_program::id(), false),

            AccountMeta::new(mint_data_account, false),

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


pub fn test(key_file: &String) -> std::result::Result<(), state::Error> {
    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let hook_program = Pubkey::from_str(HOOK_PUBKEY).unwrap();

    let mint_address = Pubkey::from_str("ERhf8aDCyZrSuAXW1fLVn78G3itxhjymGfeuY9Dgfxxa").unwrap();
    let expected_validation_address = Pubkey::from_str("2mFo1KSeohvLYzVNvxjji12F2QDN3wTVhzvN6ny37KUr").unwrap();

    let my_token_address = get_associated_token_address_with_program_id(
        &wallet.pubkey(),
        &mint_address,
        &spl_token_2022::ID,
    );

    let (mint_data_account, _old_bump_seed) = Pubkey::find_program_address(&[b"mint_data", &mint_address.to_bytes()], &hook_program);

    println!("mint data account {}",mint_data_account.to_string());

    let extra_account_data = connection.get_account_data(&expected_validation_address).unwrap();

    println!("extra_account_data {:?}",extra_account_data);
    let state = TlvStateBorrowed::unpack(&extra_account_data[..]).unwrap();
    println!("have state {:?}", state);
    let extra_meta_list = ExtraAccountMetaList::unpack_with_tlv_state::<state::ExecuteInstruction>(&state).unwrap();
    println!("have list");
    let extra_account_metas = extra_meta_list.data();

    println!("extra account metas: {:?}", extra_account_metas);

    //return Ok(());

    let new_account = Keypair::new();
    let lamports = rent::Rent::default().minimum_balance(0);

    let create_idx = solana_program::system_instruction::create_account(&wallet.pubkey(), &new_account.pubkey(), lamports, 0,&solana_sdk::system_program::id());

    let create_token_account_idx = spl_associated_token_account::instruction::create_associated_token_account(&wallet.pubkey(), &new_account.pubkey(), &mint_address, &spl_token_2022::id());

    let dest_token_address = get_associated_token_address_with_program_id(
        &new_account.pubkey(),
        &mint_address,
        &spl_token_2022::ID,
    );

    println!("dest token address: {:?}", dest_token_address.to_string());

    let mut transfer_idx = spl_token_2022::instruction::transfer_checked(
        &spl_token_2022::id(),
        &my_token_address,
        &mint_address,
        &dest_token_address,
        &wallet.pubkey(),
        &[&wallet.pubkey()],
        1000,
        3,
    ).unwrap();

    transfer_idx.accounts.push(AccountMeta::new_readonly(
        hook_program,
        false,
    ));

    transfer_idx.accounts.push(AccountMeta::new_readonly(
        expected_validation_address,
        false,
    ));

    transfer_idx.accounts.push(AccountMeta::new(
        mint_data_account,
        false,
    ));

/*
    let mut account_metas = vec![];
    spl_token_2022::offchain::get_extra_transfer_account_metas(&mut account_metas, 
        |address| connection.get_account(&address).map_ok(|opt| opt.map(|acc| acc.data)), 
        &mint_address);



        
    spl_token_2022::offchain::resolve_extra_transfer_account_metas(
        &mut instruction,
        |address| {
            rpc_client
                .get_account(address)
                .map_ok(|opt| opt.map(|acc| acc.data))
        },
        mint,
    )
    .await.unwrap();
 */



    let signers = [&wallet, &new_account];
    let instructions = vec![create_idx, create_token_account_idx, transfer_idx];
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
