use spl_associated_token_account::get_associated_token_address;
use solana_program::{pubkey::Pubkey, declare_id};
// functions to calculate expected public keys

// This can also be replaced with pubkey ("CU8AequXiVdXyVKc7Vqg2jiBDJgPwapMbcBrm7EVnTtm") if you are on a recent sdk
mod daoplays {
    use super::*;
    declare_id!("FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD");   
}


mod supporters_token_mint {
    use super::*;
    declare_id!("ESxUiMdmrZzJBk1JryJyVpD2ok9cheTV43H1HXEy8n5x");   
}


pub fn get_expected_daoplays_key() -> Pubkey
{
    daoplays::ID
}

pub fn get_expected_supporter_token_mint_key() -> Pubkey
{
    supporters_token_mint::ID
}

pub fn get_expected_daoplays_supporters_token_key() -> Pubkey
{
    get_associated_token_address(
        &get_expected_daoplays_key(), 
        &get_expected_supporter_token_mint_key()
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

pub fn get_expected_program_supporter_token_key(program_id : &Pubkey) -> Pubkey
{
    get_associated_token_address(
        &get_expected_program_address_key(program_id).0, 
        &get_expected_supporter_token_mint_key()
    )
}
