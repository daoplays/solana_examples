use borsh::{to_vec, BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::state;

#[derive(Default, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct AMM {
    pub account_type: state::AccountType,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub lp_mint: Pubkey,
    pub base_key: Pubkey,
    pub quote_key: Pubkey,
    pub fee: u16,
    pub num_data_accounts: u32,
    pub last_price: f32,
    pub lp_amount: u64,
    pub borrow_cost: u16,
    pub leverage_frac: u16,
    pub amm_base_amount: u64,
    pub amm_quote_amount: u64,
    pub short_base_amount: u64,
    pub long_quote_amount: u64,
}

pub fn get_amm_data_size() -> usize {
    let encoded = to_vec(&AMM::default()).unwrap();

    encoded.len()
}

#[derive(Default, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct OHLCV {
    pub timestamp: i64,
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f32,
}

pub fn get_candle_size() -> usize {
    let encoded = to_vec(&OHLCV::default()).unwrap();

    encoded.len()
}

#[derive(Default, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TimeSeriesData {
    pub account_type: state::AccountType,
    pub data: Vec<OHLCV>,
}

pub fn get_price_data_size() -> usize {
    let encoded = to_vec(&TimeSeriesData::default()).unwrap();

    encoded.len()
}

pub fn get_amm_seeds<'a>(base_mint: Pubkey, quote_mint: Pubkey, amm_seeds: &mut Vec<Pubkey>) {
    let base_first = base_mint.to_string() < quote_mint.to_string();

    if base_first {
        amm_seeds.push(base_mint);
        amm_seeds.push(quote_mint)
    } else {
        amm_seeds.push(quote_mint);
        amm_seeds.push(base_mint);
    }
}
