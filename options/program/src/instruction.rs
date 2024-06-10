use borsh::{BorshDeserialize, BorshSerialize};
use shank::{ShankContext, ShankInstruction};

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CreateOptionCollectionMeta {
    pub name: String,
    pub uri: String,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CreateOptionAssetMeta {
    pub name: String,
    pub uri: String,
    pub token_amount: u64,
    pub side: u8,
    pub end_date: i64,
    pub strike: u64,
    pub price: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct ListOptionAssetMeta {
    pub price: u64,
}

#[derive(
    ShankInstruction, ShankContext, BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq,
)]
pub enum OptionsInstruction {
    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "program_pda", desc = "PDA for options program")]
    #[account(2, name = "system_program", desc = "System program")]
    Init(),

    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(
        1,
        writable,
        name = "collection_account",
        desc = "Account of new collection"
    )]
    #[account(2, writable, name = "base_mint", desc = "Mint of option token")]
    #[account(3, writable, name = "quote_mint", desc = "Mint of option token")]
    #[account(4, name = "program_pda", desc = "Program PDA")]
    #[account(5, name = "core_program", desc = "MPL Core")]
    #[account(6, name = "system_program", desc = "System program")]
    CreateCollection(CreateOptionCollectionMeta),

    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "program_pda", desc = "Program PDA")]
    #[account(4, name = "base_mint", desc = "Token Mint")]
    #[account(5, name = "quote_mint", desc = "Token Mint")]
    #[account(6, name = "user_base", desc = "Users ATA")]
    #[account(7, name = "user_quote", desc = "Users ATA")]
    #[account(8, name = "program_base", desc = "Program ATA")]
    #[account(9, name = "program_quote", desc = "Program ATA")]
    #[account(10, name = "core_program", desc = "MPL Core")]
    #[account(11, name = "associated", desc = "Associated Token Program")]
    #[account(12, name = "base_token_program", desc = "Token Program")]
    #[account(13, name = "quote_token_program", desc = "Token Program")]
    #[account(14, name = "system_program", desc = "System program")]
    CreateOption(CreateOptionAssetMeta),

    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(
        1,
        writable,
        name = "destination_account",
        desc = "Account to send the NFT to"
    )]
    #[account(2, writable, name = "asset_account", desc = "Account of NFT")]
    #[account(
        3,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(4, name = "core_program", desc = "MPL Core")]
    #[account(5, name = "system_program", desc = "System program")]
    TransferToken,

    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "program_pda", desc = "Program PDA")]
    #[account(4, name = "base_mint", desc = "Token Mint")]
    #[account(5, name = "quote_mint", desc = "Token Mint")]
    #[account(6, name = "user_base", desc = "Users ATA")]
    #[account(7, name = "user_quote", desc = "Users ATA")]
    #[account(8, name = "program_base", desc = "Program ATA")]
    #[account(9, name = "program_quote", desc = "Program ATA")]
    #[account(10, name = "creator", desc = "Creator")]
    #[account(11, name = "creator_base", desc = "Creator ATA")]
    #[account(12, name = "creator_quote", desc = "Creator ATA")]
    #[account(13, name = "core_program", desc = "MPL Core")]
    #[account(14, name = "associated", desc = "Associated Token Program")]
    #[account(15, name = "base_token_program", desc = "Token Program")]
    #[account(16, name = "quote_token_program", desc = "Token Program")]
    #[account(17, name = "system_program", desc = "System program")]
    Execute(),

    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, writable, name = "program_pda", desc = "Program PDA")]
    #[account(4, name = "base_mint", desc = "Token Mint")]
    #[account(5, name = "quote_mint", desc = "Token Mint")]
    #[account(6, name = "program_base", desc = "Program ATA")]
    #[account(7, name = "program_quote", desc = "Program ATA")]
    #[account(8, writable, name = "owner", desc = "Owner")]
    #[account(9, writable, name = "creator", desc = "Creator")]
    #[account(10, name = "creator_base", desc = "Creator ATA")]
    #[account(11, name = "creator_quote", desc = "Creator ATA")]
    #[account(12, name = "core_program", desc = "MPL Core")]
    #[account(13, name = "associated", desc = "Associated Token Program")]
    #[account(14, name = "base_token_program", desc = "Token Program")]
    #[account(15, name = "quote_token_program", desc = "Token Program")]
    #[account(16, name = "system_program", desc = "System program")]
    Refund(),

    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "program_pda", desc = "Program PDA")]
    #[account(4, name = "core_program", desc = "MPL Core")]
    #[account(5, name = "system_program", desc = "System program")]
    List(ListOptionAssetMeta),

    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "program_pda", desc = "Program PDA")]
    #[account(4, name = "seller", desc = "Sellers account")]
    #[account(5, name = "core_program", desc = "MPL Core")]
    #[account(6, name = "system_program", desc = "System program")]
    Purchase(),
}
