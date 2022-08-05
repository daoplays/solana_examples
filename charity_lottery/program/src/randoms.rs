use crate::state::SeedStruct;
use std::mem;
use solana_program::{
    account_info::AccountInfo,
    msg
};
use pyth_sdk_solana::{load_price_feed_from_account_info};
use murmur3::murmur3_x64_128;



// A xorshift* generator as suggested by Marsaglia.
// The following 64-bit generator with 64 bits of state has a maximal period of 2^64−1
// and fails only the MatrixRank test of BigCrush
// see https://en.wikipedia.org/wiki/Xorshift
pub fn shift_seed(mut seed: u64) -> u64 {
    seed ^= seed >> 12;
    seed ^= seed << 25;
    seed ^= seed >> 27;
    seed *= 0x2545F4914F6CDD1D;

    return seed;

}

pub fn generate_random(seed: u64) -> f64 {

    let tmp = 0x3FF0000000000000 | (seed & 0xFFFFFFFFFFFFF);
    let result: f64 = unsafe { mem::transmute(tmp) };
    

    return result - 1.0;
}


unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::std::mem::size_of::<T>(),
    )
}

pub fn generate_seed<'a>(
    btc_account_info : &AccountInfo<'a>,
    eth_account_info : &AccountInfo<'a>,
    sol_account_info : &AccountInfo<'a>,
    ) ->u64 {

    let btc_price_feed = load_price_feed_from_account_info( &btc_account_info ).unwrap();
    let eth_price_feed = load_price_feed_from_account_info( &eth_account_info ).unwrap();
    let sol_price_feed = load_price_feed_from_account_info( &sol_account_info ).unwrap();

    let btc_price_struct = btc_price_feed.get_current_price().unwrap();
    let eth_price_struct = eth_price_feed.get_current_price().unwrap();
    let sol_price_struct = sol_price_feed.get_current_price().unwrap();

    let btc_price_value = u64::try_from(btc_price_struct.price).unwrap();
    let btc_price_error = btc_price_struct.conf;

    let eth_price_value = u64::try_from(eth_price_struct.price).unwrap();
    let eth_price_error = eth_price_struct.conf;

    let sol_price_value = u64::try_from(sol_price_struct.price).unwrap();
    let sol_price_error = sol_price_struct.conf;

    msg!("btc price: ({} +/- {}) x 10^{}", btc_price_value, btc_price_error, btc_price_struct.expo);
    msg!("eth price: ({} +/- {}) x 10^{}", eth_price_value, eth_price_error, eth_price_struct.expo);
    msg!("sol price: ({} +/- {}) x 10^{}", sol_price_value, sol_price_error, sol_price_struct.expo);

    let mut seed_values = SeedStruct { seed_prices : [0; 9] };

   
    msg!("Generating seed");    

    seed_values.seed_prices[0] = shift_seed(shift_seed(btc_price_value + btc_price_error));
    seed_values.seed_prices[1] = shift_seed(shift_seed(btc_price_value));
    seed_values.seed_prices[2] = shift_seed(shift_seed(btc_price_value - btc_price_error));

    seed_values.seed_prices[3] = shift_seed(shift_seed(eth_price_value + eth_price_error));
    seed_values.seed_prices[4] = shift_seed(shift_seed(eth_price_value));
    seed_values.seed_prices[5] = shift_seed(shift_seed(eth_price_value - eth_price_error));

    seed_values.seed_prices[6] = shift_seed(shift_seed(sol_price_value + sol_price_error));
    seed_values.seed_prices[7] = shift_seed(shift_seed(sol_price_value));
    seed_values.seed_prices[8] = shift_seed(shift_seed(sol_price_value - sol_price_error));

    let mut vec_to_hash = unsafe{any_as_u8_slice(&seed_values)};
    let h = murmur3_x64_128(&mut vec_to_hash, 0).unwrap();

    // we can take our 128bit number and get two 64bit values
    let lower  = u64::try_from(h & 0xFFFFFFFFFFFFFFFF).unwrap();
    let upper  = u64::try_from((h >> 64) & 0xFFFFFFFFFFFFFFFF).unwrap();

    let seed = lower ^ upper;
        

    return seed;
}