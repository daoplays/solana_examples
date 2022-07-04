use std::mem;
use borsh::{BorshDeserialize, BorshSerialize};
use std::str::FromStr;
use crate::state::{State, SeedStruct, RNGMeta, RNGMethod, HashStruct};
use sha2::{Sha256, Digest};
use murmur3::murmur3_x64_128;
use pyth_sdk_solana::{load_price_feed_from_account_info, PriceFeed};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,msg,
    program_error::ProgramError
};

use crate::{instruction::RNGInstruction};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        let instruction = RNGInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            RNGInstruction::GenerateRandom {metadata} => {

                Self::generate_randoms(program_id, accounts, metadata)
            }

            RNGInstruction::GenerateSeed => {

                Self::generate_seed(program_id, accounts)
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

    // create a sha256 hash from our initial seed and a nonce value to produce 4 64bit random numbers
    fn get_sha256_hashed_randoms(seed: u64, nonce: u64) -> [u64; 4] {

        let hashstruct = HashStruct {nonce : nonce, initial_seed : seed};
        let vec_to_hash = unsafe{Self::any_as_u8_slice(&hashstruct)};
        let hash= &(Sha256::new()
        .chain_update(vec_to_hash)
        .finalize()[..32]);

        // hash is a vector of 32 8bit numbers.  We can take slices of this to generate our 4 random u64s
        let mut hashed_randoms : [u64; 4] = [0; 4];
        for i in 0..4 {
            let hash_slice = &hash[i*8..(i+1)*8];
            hashed_randoms[i] = u64::from_le_bytes(hash_slice.try_into().expect("slice with incorrect length"));
        }

        return hashed_randoms;
        
    }

    // create a murmur3 hash from our initial seed and a nonce value to produce 2 64bit random numbers
    fn get_murmur_hashed_randoms(seed: u64, nonce: u64) -> [u64; 2] {

            let hashstruct = HashStruct {nonce : nonce, initial_seed : seed};
            let mut vec_to_hash = unsafe{Self::any_as_u8_slice(&hashstruct)};
            let h = murmur3_x64_128(&mut vec_to_hash, 0).unwrap();

            // we can take our 128bit number and get two 64bit values
            let lower  = u64::try_from(h & 0xFFFFFFFFFFFFFFFF).unwrap();
            let upper  = u64::try_from((h >> 64) & 0xFFFFFFFFFFFFFFFF).unwrap();
    
            let mut hashed_randoms : [u64; 2] = [0; 2];
            
            hashed_randoms[0] = lower;
            hashed_randoms[1] = upper;
            
    
            return hashed_randoms;
            
    }

    fn generate_randoms(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        meta: RNGMeta
        ) ->ProgramResult {

        let account_info_iter = &mut accounts.iter();

        // first account is the signer of this transaction
        let data_account = next_account_info(account_info_iter)?;

        let rng_creator_pubkey = Pubkey::from_str("FxVpjJ5AGY6cfCwZQP5v8QBfS4J2NPa62HbGh1Fu2LpD").unwrap();
        let correct_data_account = Pubkey::create_with_seed(
            &rng_creator_pubkey,
            "rng_v1.0",
            program_id,
        )?;

        if data_account.key != &correct_data_account
        {
            return Err(ProgramError::InvalidAccountData);
        }

        const n_randoms: usize = 256;
        let mut randoms = State { random_numbers : [0.0; n_randoms] };

        let mut seed = meta.initial_seed;
        
        match meta.method {
            RNGMethod::Xorshift => {
                msg!("Generating 256 random numbers with Xorshift method");
                for i in 0..n_randoms {
                    seed = Self::shift_seed(seed);
                    let ran = Self::generate_random_f64(seed);
                    randoms.random_numbers[i] = ran;
                }
            },
            RNGMethod::Hash => {         
                msg!("Generating 60 random numbers with SHA256 hash method");
                for i in 0..15 {
                    let nonce = i as u64;
                    let hashed_randoms = Self::get_sha256_hashed_randoms(meta.initial_seed, nonce);
                    for j in 0..4 {
                        let ran = Self::generate_random_f64(hashed_randoms[j]);
                        randoms.random_numbers[4*i + j] = ran;
                    }
                }
            }
            RNGMethod::FastHash => {         
                msg!("Generating 256 random numbers with murmur hash method");
                for i in 0..128 {
                    let nonce = i as u64;
                    let hashed_randoms = Self::get_murmur_hashed_randoms(meta.initial_seed, nonce);
                    for j in 0..2 {
                        let ran = Self::generate_random_f64(hashed_randoms[j]);
                        randoms.random_numbers[2*i + j] = ran;
                    }
                }
            }
            RNGMethod::None => {   
                msg!("Not generating random numbers to get baseline cost");
            }
        }

        randoms.serialize(&mut &mut data_account.data.borrow_mut()[..])?;


        Ok(())
    }


    fn generate_seed(
        program_id: &Pubkey,
        accounts: &[AccountInfo]
        ) ->ProgramResult {

        // we will use 3 streams, BTC,  ETH and SOL
        let BTC_key =   Pubkey::from_str("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J").unwrap();
        let ETH_key =   Pubkey::from_str("EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw").unwrap();
        let SOL_key =   Pubkey::from_str("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap();

        let account_info_iter = &mut accounts.iter();

        // first accounts are the pyth oracles
        let BTC_account_info = next_account_info(account_info_iter)?;
        let ETH_account_info = next_account_info(account_info_iter)?;
        let SOL_account_info = next_account_info(account_info_iter)?;

        // check the accounts match what we expect
        if  BTC_account_info.key != &BTC_key || 
            ETH_account_info.key != &ETH_key ||
            SOL_account_info.key != &SOL_key 
        {
            return Err(ProgramError::InvalidAccountData);
        }


        let btc_price_feed = load_price_feed_from_account_info( &BTC_account_info ).unwrap();
        let eth_price_feed = load_price_feed_from_account_info( &ETH_account_info ).unwrap();
        let sol_price_feed = load_price_feed_from_account_info( &SOL_account_info ).unwrap();

        let btc_price_struct = btc_price_feed.get_current_price().unwrap();
        let eth_price_struct = eth_price_feed.get_current_price().unwrap();
        let sol_price_struct = sol_price_feed.get_current_price().unwrap();
  
        let btc_price_value = u64::try_from(btc_price_struct.price).unwrap();
        let btc_price_error = btc_price_struct.conf;

        let eth_price_value = u64::try_from(eth_price_struct.price).unwrap();
        let eth_price_error = eth_price_struct.conf;

        let sol_price_value = u64::try_from(sol_price_struct.price).unwrap();
        let sol_price_error = sol_price_struct.conf;

        msg!("btc price: {} {}", btc_price_value, btc_price_error);
        msg!("eth price: {} {}", eth_price_value, eth_price_error);
        msg!("sol price: {} {}", sol_price_value, sol_price_error);

        let mut seed_values = SeedStruct { seed_prices : [0; 9] };
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

        let seed = lower ^ upper;
        let seed_double = Self::generate_random_f64(seed);
        msg!("final seed: {} {}", seed, seed_double);


        Ok(())
    }
}
