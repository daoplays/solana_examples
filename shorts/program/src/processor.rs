use crate::core_shorts;
use crate::{amm, instruction::accounts::InitAccounts};
use borsh::BorshDeserialize;
use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey};
use solana_program::{program::invoke_signed, program_error::ProgramError, sysvar::rent};

use crate::instruction::AMMInstruction;

pub struct Processor;
impl Processor {
    pub fn process<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Process");
        let instruction = AMMInstruction::try_from_slice(instruction_data)?;

        match instruction {
            AMMInstruction::Init() => {
                msg!("Init");
                Self::init(program_id, accounts)
            }

            AMMInstruction::InitAMM(args) => {
                msg!("Create AMM");
                amm::init_amm(program_id, accounts, args)
            }

            AMMInstruction::Swap(args) => {
                msg!("Swap");
                amm::perform_swap(program_id, accounts, args)
            }

            AMMInstruction::AddLiquidity(args) => {
                msg!("Add Liquidity");
                amm::add_liquidity(program_id, accounts, args)
            }

            AMMInstruction::RemoveLiquidity(args) => {
                msg!("Remove Liquidity");
                amm::remove_liquidity(program_id, accounts, args)
            }

            AMMInstruction::CreateCollection() => {
                msg!("Create Collection");
                core_shorts::create_collection(program_id, accounts)
            }

            AMMInstruction::EnterShort(args) => {
                msg!("Buy Short");
                core_shorts::buy_short(program_id, accounts, args)
            }
            AMMInstruction::ExitShort() => {
                msg!("Sell Short");
                core_shorts::sell_short(program_id, accounts)
            }
            AMMInstruction::LiquidateShort() => {
                msg!("Liquidate Short");
                core_shorts::liquidate_short(program_id, accounts)
            }
            AMMInstruction::EnterLong(args) => {
                msg!("Enter Long");
                core_shorts::enter_long(program_id, accounts, args)
            }
            AMMInstruction::ExitLong() => {
                msg!("Exit Long");
                core_shorts::exit_long(program_id, accounts)
            }
            AMMInstruction::LiquidateLong() => {
                msg!("Liquidate Long");
                core_shorts::liquidate_long(program_id, accounts)
            }
        }
    }

    pub fn init<'a>(program_id: &Pubkey, accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
        let ctx = InitAccounts::context(accounts)?;

        if !ctx.accounts.user.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (_pda_account, bump_seed) = Pubkey::find_program_address(&[b"pda"], &program_id);

        let lamports = rent::Rent::default().minimum_balance(0);

        let ix = solana_program::system_instruction::create_account(
            ctx.accounts.user.key,
            ctx.accounts.program_pda.key,
            lamports,
            0,
            program_id,
        );

        invoke_signed(
            &ix,
            &[ctx.accounts.user.clone(), ctx.accounts.program_pda.clone()],
            &[&[b"pda", &[bump_seed]]],
        )?;

        Ok(())
    }
}
