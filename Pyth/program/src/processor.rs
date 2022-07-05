use std::mem;
use borsh::{BorshDeserialize};
use std::str::FromStr;
use crate::state::{SeedMeta, SeedMethod, SeedStruct};
use murmur3::murmur3_x64_128;
use sha2::{Sha256, Digest};

use pyth_sdk_solana::{load_price_feed_from_account_info};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,msg,
    program_error::ProgramError
};

use crate::{instruction::SeedInstruction};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        let instruction = SeedInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            SeedInstruction::GenerateSeed {metadata} => {

                Self::generate_seed(program_id, accounts, metadata)
            }
        }
    } 
    
    // A xorshift* generator as suggested by Marsaglia.
    // The following 64-bit generator with 64 bits of state has a maximal period of 2^64−1
    // and fails only the MatrixRank test of BigCrush
    // see https://en.wikipedia.org/wiki/Xorshift
    fn shift_seed(mut seed: u64) -> u64 {
        seed ^= seed >> 12;
	    seed ^= seed << 25;
	    seed ^= seed >> 27;
	    seed *= 0x2545F4914F6CDD1D;

        return seed;

    }

    // convert the u64 into a double with range 0..1
    fn generate_random_f64(seed: u64) -> f64 {

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

    fn generate_seed(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        meta: SeedMeta
        ) ->ProgramResult {

        // we will use 3 streams, BTC,  ETH and SOL
        let btc_key =   Pubkey::from_str("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J").unwrap();
        let eth_key =   Pubkey::from_str("EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw").unwrap();
        let sol_key =   Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap();

        let account_info_iter = &mut accounts.iter();

        // first accounts are the pyth oracles
        let btc_account_info = next_account_info(account_info_iter)?;
        let eth_account_info = next_account_info(account_info_iter)?;
        let sol_account_info = next_account_info(account_info_iter)?;

        // check the accounts match what we expect
        if  btc_account_info.key != &btc_key || 
            eth_account_info.key != &eth_key ||
            sol_account_info.key != &sol_key 
        {
            return Err(ProgramError::InvalidAccountData);
        }


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

        let mut seed :  u64 = 0;
        match meta.method {
            SeedMethod::ShiftMurmur => {
                msg!("Generating seed using ShiftMurmur");    
                seed_values.seed_prices[0] = Self::shift_seed(Self::shift_seed(btc_price_value + btc_price_error));
                seed_values.seed_prices[1] = Self::shift_seed(Self::shift_seed(btc_price_value));
                seed_values.seed_prices[2] = Self::shift_seed(Self::shift_seed(btc_price_value - btc_price_error));

                seed_values.seed_prices[3] = Self::shift_seed(Self::shift_seed(eth_price_value + eth_price_error));
                seed_values.seed_prices[4] = Self::shift_seed(Self::shift_seed(eth_price_value));
                seed_values.seed_prices[5] = Self::shift_seed(Self::shift_seed(eth_price_value - eth_price_error));

                seed_values.seed_prices[6] = Self::shift_seed(Self::shift_seed(sol_price_value + sol_price_error));
                seed_values.seed_prices[7] = Self::shift_seed(Self::shift_seed(sol_price_value));
                seed_values.seed_prices[8] = Self::shift_seed(Self::shift_seed(sol_price_value - sol_price_error));

                let mut vec_to_hash = unsafe{Self::any_as_u8_slice(&seed_values)};
                let h = murmur3_x64_128(&mut vec_to_hash, 0).unwrap();

                // we can take our 128bit number and get two 64bit values
                let lower  = u64::try_from(h & 0xFFFFFFFFFFFFFFFF).unwrap();
                let upper  = u64::try_from((h >> 64) & 0xFFFFFFFFFFFFFFFF).unwrap();

                seed = lower ^ upper;
            }
            SeedMethod::SHA256Hash => {
                msg!("Generating seed using SHA256 hash");
                seed_values.seed_prices[0] = btc_price_value + btc_price_error;
                seed_values.seed_prices[1] = btc_price_value;
                seed_values.seed_prices[2] = btc_price_value - btc_price_error;
        
                seed_values.seed_prices[3] = eth_price_value + eth_price_error;
                seed_values.seed_prices[4] = eth_price_value;
                seed_values.seed_prices[5] = eth_price_value - eth_price_error;
        
                seed_values.seed_prices[6] = sol_price_value + sol_price_error;
                seed_values.seed_prices[7] = sol_price_value;
                seed_values.seed_prices[8] = sol_price_value - sol_price_error;
        
                let vec_to_hash = unsafe{Self::any_as_u8_slice(&seed_values)};
                let hash= &(Sha256::new()
                .chain_update(vec_to_hash)
                .finalize()[..32]);
        
                let hash_slice = &hash[0..8];
                seed = u64::from_le_bytes(hash_slice.try_into().expect("slice with incorrect length"));
            }
            SeedMethod::None => {
                msg!("Not generating seed to get baseline cost");
            }
        }

        let seed_double = Self::generate_random_f64(seed);
        msg!("final seed: {} => {}", seed, seed_double);

        Ok(())
    }
}
