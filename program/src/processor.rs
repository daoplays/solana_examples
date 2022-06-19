use std::mem;
use borsh::{BorshDeserialize, BorshSerialize};
use std::str::FromStr;
use crate::state::{State};


use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    pubkey::Pubkey,
    program_error::ProgramError,
};

use crate::{instruction::RNGInstruction};

pub struct Processor;
impl Processor {
    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {

        let instruction = RNGInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            RNGInstruction::GenerateRandom {initial_seed} => {

                Self::generate_randoms(program_id, accounts, initial_seed)
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
    fn generate_random(seed: u64) -> f64 {

        let tmp = 0x3FF0000000000000 | (seed & 0xFFFFFFFFFFFFF);
        let result: f64 = unsafe { mem::transmute(tmp) };
        
        return result - 1.0;
    }

    fn generate_randoms(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        initial_seed: u64
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

        let mut randoms = State::try_from_slice(&data_account.data.borrow()[..])?;

        let n_randoms = 512;
        let mut seed = initial_seed;
        for i in 0..n_randoms {
            seed = Self::shift_seed(seed);
            let ran = Self::generate_random(seed);
            randoms.random_numbers[i] = ran;
        }

        randoms.serialize(&mut &mut data_account.data.borrow_mut()[..])?;


        Ok(())
    }
}