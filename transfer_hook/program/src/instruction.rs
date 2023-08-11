use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

use crate::error::NewError::InvalidInstruction;

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
    /// Initializes the extra account metas on an account, writing into
    /// the first open TLV space.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[w]` Account with extra account metas
    ///   1. `[]` Mint
    ///   2. `[s]` Mint authority
    ///   3. `[]` System program
    ///   4..4+M `[]` `M` additional accounts, to be written to validation data
    ///
    InitializeExtraAccountMetas,
}

impl TransferHookInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::Execute {
                amount: u64::try_from_slice(&rest)?,
            },
            1 => Self::InitializeExtraAccountMetas,

            _ => return Err(InvalidInstruction.into()),
        })
    }
}
