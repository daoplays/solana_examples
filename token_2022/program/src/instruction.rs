use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::error::NewError::InvalidInstruction;
use spl_discriminator::SplDiscriminate;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CreateMeta {
    pub extensions: u8,
    pub transfer_fee_bp: u16,
    pub transfer_fee_max: u64,
    pub interest_rate: i16,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct TransferMeta {
    pub amount: u64
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TokenInstruction {
    CreateToken { metadata: CreateMeta },
    Transfer { metadata: TransferMeta },
}

impl TokenInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::CreateToken {
                metadata: CreateMeta::try_from_slice(&rest)?,
            },
            1 => Self::Transfer {
                metadata: TransferMeta::try_from_slice(&rest)?,
            },

            _ => return Err(InvalidInstruction.into()),
        })
    }
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TransferHookInstruction {
    /// Runs additional transfer logic.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` Source account
    ///   1. `[]` Token mint
    ///   2. `[]` Destination account
    ///   3. `[]` Source account's owner/delegate
    ///   4. `[]` Validation account
    ///   5..5+M `[]` `M` additional accounts, written in validation account data
    ///
    Execute {
        /// Amount of tokens to transfer
        amount: u64,
    },
    InitializeExtraAccountMetas
}

/// TLV instruction type only used to define the discriminator. The actual data
/// is entirely managed by `ExtraAccountMetaList`, and it is the only data contained
/// by this type.
#[derive(SplDiscriminate)]
#[discriminator_hash_input("spl-transfer-hook-interface:execute")]
pub struct ExecuteInstruction;

/// TLV instruction type used to initialize extra account metas
/// for the transfer hook
#[derive(SplDiscriminate)]
#[discriminator_hash_input("spl-transfer-hook-interface:initialize-extra-account-metas")]
pub struct InitializeExtraAccountMetaListInstruction;



impl TransferHookInstruction {

    /// Packs a [TokenInstruction](enum.TokenInstruction.html) into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        let mut buf = vec![];
        match self {
            Self::Execute { amount } => {
                buf.extend_from_slice(ExecuteInstruction::SPL_DISCRIMINATOR_SLICE);
                buf.extend_from_slice(&amount.to_le_bytes());
            },
            Self::InitializeExtraAccountMetas => {
                buf.extend_from_slice(InitializeExtraAccountMetaListInstruction::SPL_DISCRIMINATOR_SLICE);
            }
            
        };
        buf
    }
}

