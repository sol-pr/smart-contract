use ahash::{HashMap, HashMapExt};
use borsh::{BorshDeserialize, BorshSerialize};
use borsh_derive::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct User {
    pub github_username: String,
    pub phantom_wallet: [u8; 32],
    pub totalearn: u64,    // toplam kazanc
    pub submitted_at: u64, // kullanicinin bu hafta yaptigi pr sayisi
    pub total_pr_count: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct UserForCreate {
    pub github_username: String,
    pub phantom_wallet: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct GithubRepo {
    pub id: String,                     //+
    pub repo_url: String,               //+
    pub repo_name: String,              //+
    pub repo_description: String,       //+
    pub total_pull_requests: u64,       // populer repolaro belirlerim
    pub pull_request_limit: u64,        //+
    pub reward_per_pull_request: u64,   // Her pull request için ödül miktarı //+
    pub owner_wallet_address: [u8; 32], // repo sahibinin cuzdan adresi //+
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct PrCount {
    pub prcount: u64,
}


#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct CheckTransfer {
    pub github_username: String,
    pub id: String, 
 }