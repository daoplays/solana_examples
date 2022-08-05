use spl_associated_token_account::get_associated_token_address;
use solana_program::{pubkey::Pubkey, declare_id};
use std::str::FromStr;
// functions to calculate expected public keys

mod btc_oracle {
    use super::*;
    declare_id!("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J");   
}
mod eth_oracle {
    use super::*;
    declare_id!("EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw");   
}
mod sol_oracle {
    use super::*;
    declare_id!("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix");   
}
// This can also be replaced with pubkey ("CU8AequXiVdXyVKc7Vqg2jiBDJgPwapMbcBrm7EVnTtm") if you are on a recent sdk
mod daoplays {
    use super::*;
    declare_id!("2BLkynLAWGwW58SLDAnhwsoiAuVtzqyfHKA3W3MJFwEF");   
}

mod token_mint {
    use super::*;
    declare_id!("CisHceikLeKxYiUqgDVduw2py2GEK71FTRykXGdwf22h");   
}

pub fn get_expected_btc_key() -> Pubkey
{
    btc_oracle::ID
}

pub fn get_expected_eth_key() -> Pubkey
{
    eth_oracle::ID
}

pub fn get_expected_sol_key() -> Pubkey
{
    sol_oracle::ID
}

pub fn get_expected_daoplays_key() -> Pubkey
{
    daoplays::ID
}

pub fn get_expected_token_mint_key() -> Pubkey
{
    token_mint::ID
}

pub fn get_expected_daoplays_token_key() -> Pubkey
{
    get_associated_token_address(
        &get_expected_daoplays_key(), 
        &get_expected_token_mint_key()
    )
}

pub fn get_pda_bump() -> u8
{
    255
}

pub fn get_expected_program_address_key(program_id : &Pubkey) -> (Pubkey, u8)
{
    let program_address = Pubkey::create_program_address(&[b"token_account", &[get_pda_bump()]], &program_id).unwrap();

    (program_address, get_pda_bump())
}

pub fn get_expected_data_account_key(program_id : &Pubkey) -> Pubkey
{
    let data_key = Pubkey::create_with_seed(
        &get_expected_daoplays_key(),
        "data_account",
        &program_id,
    ).unwrap();

    return data_key;

}

pub fn get_expected_program_token_key(program_id : &Pubkey) -> Pubkey
{
    get_associated_token_address(
        &get_expected_program_address_key(program_id).0, 
        &get_expected_token_mint_key()
    )
}
