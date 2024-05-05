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
    #[account(2, writable, name = "token_mint", desc = "Mint of option token")]
    #[account(3, name = "program_pda", desc = "Program PDA")]
    #[account(4, name = "core_program", desc = "MPL Core")]
    #[account(5, name = "system_program", desc = "System program")]
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
    #[account(4, name = "token_mint", desc = "Token Mint")]
    #[account(5, name = "user_token_account", desc = "Users ATA")]
    #[account(6, name = "program_token_account", desc = "Program ATA")]
    #[account(7, name = "core_program", desc = "MPL Core")]
    #[account(8, name = "token_2022", desc = "Token 2022 Program")]
    #[account(9, name = "system_program", desc = "System program")]
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
    #[account(4, name = "token_mint", desc = "Token Mint")]
    #[account(5, name = "user_token_account", desc = "Users ATA")]
    #[account(6, name = "program_token_account", desc = "Program ATA")]
    #[account(7, name = "creator", desc = "Creator")]
    #[account(8, name = "creator_token_account", desc = "Creator ATA")]
    #[account(9, name = "core_program", desc = "MPL Core")]
    #[account(10, name = "token_2022", desc = "Token 2022 Program")]
    #[account(11, name = "system_program", desc = "System program")]
    Execute(),

    #[account(0, writable, signer, name = "user", desc = "Users account, signer")]
    #[account(1, writable, name = "asset_account", desc = "Account of new NFT")]
    #[account(
        2,
        writable,
        name = "collection_account",
        desc = "Account of NFT collection"
    )]
    #[account(3, name = "program_pda", desc = "Program PDA")]
    #[account(4, name = "token_mint", desc = "Token Mint")]
    #[account(5, name = "program_token_account", desc = "Program ATA")]
    #[account(6, name = "creator", desc = "Creator")]
    #[account(7, name = "creator_token_account", desc = "Creator ATA")]
    #[account(8, name = "core_program", desc = "MPL Core")]
    #[account(9, name = "token_2022", desc = "Token 2022 Program")]
    #[account(10, name = "system_program", desc = "System program")]
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
}
