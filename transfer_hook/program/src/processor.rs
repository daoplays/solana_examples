use crate::state;
use crate::utils;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::rent,
    instruction::AccountMeta
};

use spl_type_length_value::state::TlvStateBorrowed;

use crate::instruction::{TransferHookInstruction, ExecuteInstruction};
use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList, seeds::Seed};


pub struct Processor;
impl Processor {
    /// Processes an [Instruction](enum.Instruction.html).
    pub fn process<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {
       // let instruction = TransferHookInstruction::try_from_slice(&instruction_data[..])?;

       msg!("unpack");
        let instruction = TransferHookInstruction::unpack(instruction_data)?;

        match instruction {
            TransferHookInstruction::Execute { amount } => {
                msg!("Instruction: Execute");
                Self::process_execute(program_id, accounts, amount)
            }
            TransferHookInstruction::InitializeExtraAccountMetaList  => {
                msg!("Instruction: InitializeExtraAccountMetaList");
                Self::process_initialize_extra_account_metas(program_id, accounts)
            }
        }
    }

    /// Processes an [Execute](enum.TransferHookInstruction.html) instruction.
    pub fn process_execute<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        _amount: u64,
    ) -> ProgramResult {

        let account_info_iter = &mut accounts.iter();

        msg!("in execute");

        
        let _source_account_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let _destination_account_info = next_account_info(account_info_iter)?;
        let _authority_info = next_account_info(account_info_iter)?;
        let extra_account_metas_info = next_account_info(account_info_iter)?;

        // For the example program, we just check that the correct pda and validation
        // pubkeys are provided
        let expected_validation_address = utils::get_extra_account_metas_address(mint_info.key, program_id);
        if expected_validation_address != *extra_account_metas_info.key {
            return Err(ProgramError::InvalidSeeds);
        }

        let data = extra_account_metas_info.data.borrow();

        let state = TlvStateBorrowed::unpack(&data[..]).unwrap();
        let extra_meta_list = ExtraAccountMetaList::unpack_with_tlv_state::<ExecuteInstruction>(&state)?;
        let extra_account_metas = extra_meta_list.data();

        msg!("Have {} metas", extra_account_metas.len());

        if extra_account_metas.len() > 0 {
            let meta = extra_account_metas[0];
            msg!("meta {:?}", meta);

            let mint_data_account_info = next_account_info(account_info_iter)?;


            let mut player_state =
            state::MintData::try_from_slice(&mint_data_account_info.data.borrow()[..])?;

            msg!("data {:?}", player_state);

            player_state.count += 1;

            player_state.serialize(&mut &mut mint_data_account_info.data.borrow_mut()[..])?;


        }
        
        Ok(())
    }


    pub fn create_program_account<'a>(
        funding_account: &AccountInfo<'a>,
        pda: &AccountInfo<'a>,
        program_id: &Pubkey,
        bump_seed: u8,
        data_size: usize,
        seed: Vec<&[u8]>,
    ) -> ProgramResult {
        // Check if the account has already been initialized
        if **pda.try_borrow_lamports()? > 0 {
            msg!("This account is already initialized. skipping");
            return Ok(());
        }

        msg!("Creating program derived account");

        let space: u64 = data_size.try_into().unwrap();
        let lamports = rent::Rent::default().minimum_balance(data_size);

        msg!("Require {} lamports for {} size data", lamports, data_size);
        let ix = solana_program::system_instruction::create_account(
            funding_account.key,
            pda.key,
            lamports,
            space,
            program_id,
        );

        // Sign and submit transaction
        if seed.len() == 1 {
            // Sign and submit transaction
            invoke_signed(
                &ix,
                &[funding_account.clone(), pda.clone()],
                &[&[seed[0], &[bump_seed]]],
            )?;
        }

        if seed.len() == 2 {
            // Sign and submit transaction
            invoke_signed(
                &ix,
                &[funding_account.clone(), pda.clone()],
                &[&[seed[0], seed[1], &[bump_seed]]],
            )?;
        }

        Ok(())
    }



    pub fn process_initialize_extra_account_metas<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let extra_account_metas_info = next_account_info(account_info_iter)?;
        let mint_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        let _system_program_info = next_account_info(account_info_iter)?;

        // we may additionally pass an account to hold data for the mint
        let mint_data_option: Option<&AccountInfo<'_>> = account_info_iter.next();

        // Check signers
        if !authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Check validation account
        let (expected_validation_address, bump_seed) =
            utils::get_extra_account_metas_address_and_bump_seed(mint_info.key, program_id);

        if expected_validation_address != *extra_account_metas_info.key {
            return Err(ProgramError::InvalidSeeds);
        }

        let mut n_extra_accounts = 0;

        let mut extra_account_infos : Vec<ExtraAccountMeta> = vec![];
        // if we did pass a mint_data account then create that now
        if mint_data_option.is_some() {

            msg!("Create mint data account");
            let mint_data_account_info = mint_data_option.unwrap();

            let bump_seed = utils::check_program_data_account(mint_data_account_info, program_id,vec![b"mint_data", &mint_info.key.to_bytes()], "mint_data".to_string()).unwrap();
            
            let data_size = state::get_mint_data_size();
            Self::create_program_account(authority_info, mint_data_account_info, program_id, bump_seed, data_size, vec![b"mint_data", &mint_info.key.to_bytes()]).unwrap();

            let seed1 = Seed::Literal { bytes: b"mint_data".to_vec()};
            let seed2 = Seed::AccountKey { index: 1 };

            msg!("seed sizes {}", seed1.tlv_size());
            
            msg!("create mint data extra meta");

            let mint_account_meta = ExtraAccountMeta::new_with_seeds(&[seed1, seed2],false, true).unwrap();
            extra_account_infos.push(mint_account_meta);

            n_extra_accounts = 1;
        }
        

        let account_size = ExtraAccountMetaList::size_of(n_extra_accounts)?;

        msg!("Have meta list of size {} for {} accounts", account_size, n_extra_accounts);

        let lamports = rent::Rent::default().minimum_balance(account_size);

        msg!("Require {} lamports for {} size data", lamports, account_size);
        let ix = solana_program::system_instruction::create_account(
            authority_info.key,
            extra_account_metas_info.key,
            lamports,
            account_size as u64,
            program_id,
        );

        // Sign and submit transaction
        invoke_signed(
            &ix,
            &[authority_info.clone(), extra_account_metas_info.clone()],
            &[&[utils::EXTRA_ACCOUNT_METAS_SEED, mint_info.key.as_ref(), &[bump_seed]]],
        )?;

        msg!("init extra account meta");
        let mut data = extra_account_metas_info.try_borrow_mut_data()?;
        if  n_extra_accounts == 0 {
            ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &[])?;
        }
        else {
            ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &[extra_account_infos[0]])?;
        }


        Ok(())
    }
}
