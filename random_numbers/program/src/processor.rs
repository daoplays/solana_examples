use std::mem;
use borsh::{BorshDeserialize, BorshSerialize};
use std::str::FromStr;
use crate::state::{State, RNGMeta, RNGMethod, HashStruct};
use sha2::{Sha256, Digest};

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
    fn get_hashed_randoms(seed: u64, nonce: u64) -> [u64; 4] {

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
                msg!("Generating 60 random numbers with hash method");
                for i in 0..15 {
                    let nonce = i as u64;
                    let hashed_randoms = Self::get_hashed_randoms(meta.initial_seed, nonce);
                    for j in 0..4 {
                        let ran = Self::generate_random_f64(hashed_randoms[j]);
                        randoms.random_numbers[4*i + j] = ran;
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
}