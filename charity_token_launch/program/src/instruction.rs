use solana_program::program_error::ProgramError;
use borsh::{BorshDeserialize, BorshSerialize};

use crate::error::RNGError::InvalidInstruction;
use crate::state::{JoinMeta, InitMeta};



#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub enum TokenLaunchInstruction {

    // Function that sets up the token launch and initialises program data and transfers tokens, expects 11 accounts:
    //funding_account_info
    //program_derived_account_info

    //token_source_account_info
    //program_token_account_info
    //token_mint_account_info

    //supporters_token_source_account_info
    //program_supporters_token_account_info
    //supporters_token_mint_account_info

    //token_program_account_info
    //associated_token_account_info
    //system_program_account_info
    InitTokenLaunch {
        metadata: InitMeta
    },


    // function that allows a user to participate in the token launch.  Sends SOL to the charity and developers and tokens to the user
    // expects 13 accounts to be passed to the function:
    //joiner_account_info
    //joiner_token_account_info
    //joiner_supporters_token_account_info
   
    //program_data_account_info
    //program_token_account_info
    //program_supporters_token_account_info
    
    //charity_account_info
    //daoplays_account_info

    //token_mint_account_info
    //supporters_token_mint_account_info

    //token_program_account_info
    //associated_token_account_info
    //system_program_account_info

    JoinTokenLaunch {
        metadata: JoinMeta
    },

    // function to end the token launch and transfer remaining tokens away from the program
    // expects 10 accounts to be passed
    //daoplays_account_info
    //daoplays_token_account_info
    //daoplays_supporters_token_account_info

    //program_account_info
    //program_token_account_info
    //program_supporters_token_account_info

    //token_mint_account_info
    //supporters_token_mint_account_info

    //token_program_account_info
    //system_program_account_info

    EndTokenLaunch
}

impl TokenLaunchInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction].
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {

            0 => Self::InitTokenLaunch {
                metadata: InitMeta::try_from_slice(&rest)?,
            },
            1 => Self::JoinTokenLaunch {
                metadata: JoinMeta::try_from_slice(&rest)?,
            },
            2 => Self::EndTokenLaunch,
            _ => return Err(InvalidInstruction.into()),
        })
    }
}