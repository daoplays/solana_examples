use crate::utils;
use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instruction::{CreateMeta, TokenInstruction};
use mpl_core::instructions::{CreateCollectionV1CpiBuilder, CreateV1CpiBuilder};
pub struct Processor;
impl Processor {
    pub fn process<'a>(
        program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = TokenInstruction::try_from_slice(&instruction_data[..])?;

        match instruction {
            TokenInstruction::CreateCollection { args } => {
                Self::create_collection(program_id, accounts, args)
            }

            TokenInstruction::CreateToken { args } => {
                Self::create_token(program_id, accounts, args)
            }
            TokenInstruction::TransferToken => utils::transfer(program_id, accounts),
        }
    }

    fn create_collection<'a>(
        _program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        args: CreateMeta,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_account_info = next_account_info(account_info_iter)?;
        let asset_account_info: &AccountInfo<'a> = next_account_info(account_info_iter)?;
        let core_program_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        let _create_cpi = CreateCollectionV1CpiBuilder::new(core_program_account_info)
            .collection(asset_account_info)
            .payer(user_account_info)
            .update_authority(None)
            .name(args.name.to_string())
            .uri(args.uri.to_string())
            .system_program(system_program_account_info)
            .invoke()
            .unwrap();

        mpl_core::instructions::AddCollectionPluginV1CpiBuilder::new(core_program_account_info)
            .collection(asset_account_info)
            .payer(user_account_info)
            .authority(Some(user_account_info))
            .plugin(mpl_core::types::Plugin::Attributes(
                mpl_core::types::Attributes {
                    attribute_list: Vec::new(),
                },
            ))
            .system_program(system_program_account_info)
            .invoke()
            .unwrap();

        Ok(())
    }

    fn create_token<'a>(
        _program_id: &Pubkey,
        accounts: &'a [AccountInfo<'a>],
        args: CreateMeta,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_account_info = next_account_info(account_info_iter)?;
        let asset_account_info: &AccountInfo<'a> = next_account_info(account_info_iter)?;
        let collection_account_info: &AccountInfo<'a> = next_account_info(account_info_iter)?;
        let core_program_account_info = next_account_info(account_info_iter)?;
        let system_program_account_info = next_account_info(account_info_iter)?;

        let _create_cpi = CreateV1CpiBuilder::new(core_program_account_info)
            .authority(Some(user_account_info))
            .asset(asset_account_info)
            .collection(Some(collection_account_info))
            .payer(user_account_info)
            .owner(Some(user_account_info))
            .data_state(mpl_core::types::DataState::AccountState)
            .name(args.name.to_string())
            .uri(args.uri.to_string())
            .system_program(system_program_account_info)
            .invoke()
            .unwrap();

        let collection =
            mpl_core::Collection::from_bytes(&collection_account_info.data.borrow()[..])?;

        let collection_size = collection.base.num_minted;

        msg!("collection size {}", collection_size);

        mpl_core::instructions::AddPluginV1CpiBuilder::new(core_program_account_info)
            .asset(asset_account_info)
            .collection(Some(collection_account_info))
            .payer(user_account_info)
            .authority(Some(user_account_info))
            .plugin(mpl_core::types::Plugin::Attributes(
                mpl_core::types::Attributes {
                    attribute_list: vec![mpl_core::types::Attribute {
                        key: "index".to_string(),
                        value: collection_size.to_string(),
                    }],
                },
            ))
            .system_program(system_program_account_info)
            .invoke()
            .unwrap();

        let mut collection_attributes = collection
            .plugin_list
            .attributes
            .unwrap()
            .attributes
            .attribute_list;

        collection_attributes.push(mpl_core::types::Attribute {
            key: asset_account_info.key.to_string(),
            value: collection_size.to_string(),
        });

        mpl_core::instructions::UpdateCollectionPluginV1CpiBuilder::new(core_program_account_info)
            .collection(collection_account_info)
            .authority(Some(user_account_info))
            .payer(user_account_info)
            .plugin(mpl_core::types::Plugin::Attributes(
                mpl_core::types::Attributes {
                    attribute_list: collection_attributes,
                },
            ))
            .system_program(system_program_account_info)
            .invoke()
            .unwrap();

        Ok(())
    }
}
