use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CreateOptionCollectionMeta {
    pub name: String,
    pub uri: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct InitAMMArgs {
    pub base_quantity: u64,
    pub quote_quantity: u64,
    pub fee: u16,
    pub short_fraction: u16,
    pub borrow_fee: u16,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct SwapArgs {
    pub side: u8,
    pub in_amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct EnterPositionArgs {
    pub borrow_amount: u64,
    pub overhead: u64,
}

#[derive(
    ShankInstruction, ShankContext, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq,
)]
pub enum AMMInstruction {
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "program_pda", desc = "PDA for program")]
    #[account(2, name = "system_program", desc = "System program")]
    Init(),

    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "pda", desc = "lets cook PDA")]
    #[account(2, writable, name = "amm", desc = "AMM data account")]
    #[account(3, writable, name = "base_token_mint", desc = "base token mint")]
    #[account(4, writable, name = "quote_token_mint", desc = "quote token mint")]
    #[account(5, writable, name = "lp_token_mint", desc = "AMM LP token mint")]
    #[account(6, writable, name = "user_base", desc = "user base account")]
    #[account(7, writable, name = "user_quote", desc = "user quote account")]
    #[account(8, writable, name = "user_lp", desc = "user LP account")]
    #[account(9, writable, name = "amm_base", desc = "user base account")]
    #[account(10, writable, name = "amm_quote", desc = "AMM quote account")]
    #[account(11, writable, name = "price_data", desc = "Price Data for AMM")]
    #[account(12, name = "quote_token_program", desc = "token program for quote")]
    #[account(13, name = "base_token_program", desc = "token progrma for base token")]
    #[account(14, name = "associated_token", desc = "associated token program")]
    #[account(15, name = "system_program", desc = "system program")]
    InitAMM(InitAMMArgs),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "user_base", desc = "user base ATA")]
    #[account(2, writable, name = "user_quote", desc = "user quote ATA")]
    #[account(3, writable, name = "base_token_mint", desc = "base token mint")]
    #[account(4, writable, name = "quote_token_mint", desc = "quote token mint")]
    #[account(5, writable, name = "amm", desc = "AMM data account")]
    #[account(6, writable, name = "amm_base", desc = "AMM base account")]
    #[account(7, writable, name = "amm_quote", desc = "AMM quote account")]
    #[account(8, writable, name = "price_data", desc = "Price Data for AMM")]
    #[account(9, writable, name = "pda", desc = "PDA for AMM")]
    #[account(10, name = "quote_token_program", desc = "token program for quote")]
    #[account(11, name = "base_token_program", desc = "token progrma for base token")]
    #[account(12, name = "associated_token", desc = "associated token program")]
    #[account(13, name = "system_program", desc = "system program")]
    Swap(SwapArgs),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "base_token_mint", desc = "base token mint")]
    #[account(2, writable, name = "quote_token_mint", desc = "quote token mint")]
    #[account(3, writable, name = "lp_token_mint", desc = "AMM LP token mint")]
    #[account(4, writable, name = "user_base", desc = "user base account")]
    #[account(5, writable, name = "user_quote", desc = "user quote account")]
    #[account(6, writable, name = "user_lp", desc = "user LP account")]
    #[account(7, writable, name = "amm", desc = "AMM data account")]
    #[account(8, writable, name = "amm_base", desc = "AMM base account")]
    #[account(9, writable, name = "amm_quote", desc = "AMM quote account")]
    #[account(10, writable, name = "pda", desc = "lets cook PDA")]
    #[account(11, name = "quote_token_program", desc = "token program for quote")]
    #[account(12, name = "base_token_program", desc = "token progrma for base token")]
    #[account(13, name = "associated_token", desc = "associated token program")]
    #[account(14, name = "system_program", desc = "system program")]
    AddLiquidity(SwapArgs),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "base_token_mint", desc = "base token mint")]
    #[account(2, writable, name = "quote_token_mint", desc = "quote token mint")]
    #[account(3, writable, name = "lp_token_mint", desc = "AMM LP token mint")]
    #[account(4, writable, name = "user_base", desc = "user base account")]
    #[account(5, writable, name = "user_quote", desc = "user quote account")]
    #[account(6, writable, name = "user_lp", desc = "user LP account")]
    #[account(7, writable, name = "amm", desc = "AMM data account")]
    #[account(8, writable, name = "amm_base", desc = "AMM base account")]
    #[account(9, writable, name = "amm_quote", desc = "AMM quote account")]
    #[account(10, writable, name = "pda", desc = "lets cook PDA")]
    #[account(11, name = "quote_token_program", desc = "token program for quote")]
    #[account(12, name = "base_token_program", desc = "token progrma for base token")]
    #[account(13, name = "associated_token", desc = "associated token program")]
    #[account(14, name = "system_program", desc = "system program")]
    RemoveLiquidity(SwapArgs),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(
        1,
        writable,
        name = "collection_account",
        desc = "Account of new collection"
    )]
    #[account(2, name = "amm", desc = "address of amm")]
    #[account(3, name = "pda", desc = "Program PDA")]
    #[account(4, name = "core_program", desc = "MPL Core")]
    #[account(5, name = "system_program", desc = "System program")]
    CreateCollection(),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "pda", desc = "Program PDA")]
    #[account(4, writable, name = "amm", desc = "AMM account")]
    #[account(5, name = "base_mint", desc = "Base Mint")]
    #[account(6, name = "quote_mint", desc = "Base Mint")]
    #[account(7, writable, name = "user_quote", desc = "Users ATA")]
    #[account(8, writable, name = "amm_quote", desc = "AMM Quote ATA")]
    #[account(9, writable, name = "price_data", desc = "Price Data for AMM")]
    #[account(10, name = "core_program", desc = "MPL Core")]
    #[account(11, name = "quote_token_program", desc = "token program for quote")]
    #[account(12, name = "base_token_program", desc = "token progrma for base token")]
    #[account(13, name = "associated_token", desc = "associated token program")]
    #[account(14, name = "system_program", desc = "System program")]
    EnterShort(EnterPositionArgs),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "pda", desc = "Program PDA")]
    #[account(4, writable, name = "amm", desc = "AMM account")]
    #[account(5, name = "base_mint", desc = "Base Mint")]
    #[account(6, name = "quote_mint", desc = "Base Mint")]
    #[account(7, writable, name = "user_quote", desc = "Users ATA")]
    #[account(8, writable, name = "amm_quote", desc = "AMM quote ATA")]
    #[account(9, writable, name = "price_data", desc = "Price Data for AMM")]
    #[account(10, name = "core_program", desc = "MPL Core")]
    #[account(11, name = "quote_token_program", desc = "token program for quote")]
    #[account(12, name = "base_token_program", desc = "token progrma for base token")]
    #[account(13, name = "associated_token", desc = "associated token program")]
    #[account(14, name = "system_program", desc = "System program")]
    ExitShort(),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "pda", desc = "Program PDA")]
    #[account(4, writable, name = "amm", desc = "AMM account")]
    #[account(5, name = "base_mint", desc = "Base Mint")]
    #[account(6, name = "quote_mint", desc = "Base Mint")]
    #[account(7, writable, name = "user_quote", desc = "Users ATA")]
    #[account(8, writable, name = "amm_quote", desc = "AMM quote ATA")]
    #[account(9, writable, name = "price_data", desc = "Price Data for AMM")]
    #[account(10, name = "core_program", desc = "MPL Core")]
    #[account(11, name = "quote_token_program", desc = "token program for quote")]
    #[account(12, name = "base_token_program", desc = "token progrma for base token")]
    #[account(13, name = "associated_token", desc = "associated token program")]
    #[account(14, name = "system_program", desc = "System program")]
    LiquidateShort(),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "pda", desc = "Program PDA")]
    #[account(4, writable, name = "amm", desc = "AMM account")]
    #[account(5, name = "base_mint", desc = "Base Mint")]
    #[account(6, name = "quote_mint", desc = "Base Mint")]
    #[account(7, writable, name = "user_base", desc = "Users Base ATA")]
    #[account(8, writable, name = "amm_base", desc = "AMM Base ATA")]
    #[account(9, writable, name = "price_data", desc = "Price Data for AMM")]
    #[account(10, name = "core_program", desc = "MPL Core")]
    #[account(11, name = "quote_token_program", desc = "token program for quote")]
    #[account(12, name = "base_token_program", desc = "token progrma for base token")]
    #[account(13, name = "associated_token", desc = "associated token program")]
    #[account(14, name = "system_program", desc = "System program")]
    EnterLong(EnterPositionArgs),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "pda", desc = "Program PDA")]
    #[account(4, writable, name = "amm", desc = "AMM account")]
    #[account(5, name = "base_mint", desc = "Base Mint")]
    #[account(6, name = "quote_mint", desc = "Base Mint")]
    #[account(7, writable, name = "user_base", desc = "Users base ATA")]
    #[account(8, writable, name = "amm_base", desc = "AMM base ATA")]
    #[account(9, writable, name = "price_data", desc = "Price Data for AMM")]
    #[account(10, name = "core_program", desc = "MPL Core")]
    #[account(11, name = "quote_token_program", desc = "token program for quote")]
    #[account(12, name = "base_token_program", desc = "token progrma for base token")]
    #[account(13, name = "associated_token", desc = "associated token program")]
    #[account(14, name = "system_program", desc = "System program")]
    ExitLong(),
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "pda", desc = "Program PDA")]
    #[account(4, writable, name = "amm", desc = "AMM account")]
    #[account(5, name = "base_mint", desc = "Base Mint")]
    #[account(6, name = "quote_mint", desc = "Base Mint")]
    #[account(7, writable, name = "user_quote", desc = "Users ATA")]
    #[account(8, writable, name = "amm_quote", desc = "AMM quote ATA")]
    #[account(9, writable, name = "price_data", desc = "Price Data for AMM")]
    #[account(10, name = "core_program", desc = "MPL Core")]
    #[account(11, name = "quote_token_program", desc = "token program for quote")]
    #[account(12, name = "base_token_program", desc = "token progrma for base token")]
    #[account(13, name = "associated_token", desc = "associated token program")]
    #[account(14, name = "system_program", desc = "System program")]
    LiquidateLong(),
}
