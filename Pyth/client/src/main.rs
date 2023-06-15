pub mod state;

use std::env;
use std::str::FromStr;
use crate::state::{Result};

use solana_client::rpc_client::RpcClient;
use solana_program::{pubkey::Pubkey};
use solana_sdk::{
    signer::Signer,
    instruction::{AccountMeta, Instruction},
    transaction::Transaction, signer::keypair::read_keypair_file,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_transaction_status::UiTransactionEncoding;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum SeedMethod {
    ShiftMurmur,
    SHA256Hash,
    None
}

fn get_method_from_int(index: u64) -> SeedMethod {
    if index == 0 {
        return SeedMethod::ShiftMurmur;
    } else  if index == 1 {
        return SeedMethod::SHA256Hash;
    }
    else {
        return SeedMethod::None;
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct SeedMeta {
    pub method : SeedMethod
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum RNGInstruction {

    GenerateSeed {
        metadata : SeedMeta
    }
}
const URL: &str = "https://api.devnet.solana.com";


fn main() {

    let args: Vec<String> = env::args().collect();
    let key_file = &args[1];
    let function = &args[2];

    if function == "generate_seed" {
        let index_arg = &args[3];
        let index: u64 = index_arg.parse().unwrap();
        let method = get_method_from_int(index);
        if let Err(err) = generate_seed(key_file, method) {
            eprintln!("{:?}", err);
            std::process::exit(1);
        }
    }
    

}

fn generate_seed(key_file: &String, method: SeedMethod) ->Result<()> {

    // (2) Create a new Keypair for the new account
    let wallet = read_keypair_file(key_file).unwrap();

    // (3) Create RPC client to be used to talk to Solana cluster
    let connection = RpcClient::new(URL);

    let program = Pubkey::from_str("Hqw9GzaxEg1efH8BciNN5D32A5fMAfBdDM3qudRdb9o5").unwrap();
    let btc_key = Pubkey::from_str("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J").unwrap();
    let eth_key = Pubkey::from_str("EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw").unwrap();
    let sol_key = Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap();


    let meta_data =  SeedMeta{method: method};

    let gen_seed_idx = Instruction::new_with_borsh(
        program,
        &RNGInstruction::GenerateSeed{metadata : meta_data},
        vec![
            AccountMeta::new_readonly(btc_key, false),
            AccountMeta::new_readonly(eth_key, false),
            AccountMeta::new_readonly(sol_key, false)
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
