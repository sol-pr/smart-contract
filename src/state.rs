use ahash::{HashMap, HashMapExt};
use borsh::{BorshDeserialize, BorshSerialize};
use borsh_derive::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct User{
    pub github_username: String, 
    pub phantom_wallet:[u8; 32], 
    pub total_pull_request_count: u32, // kullanicinin yaptigi PR sayisi
    pub pull_requests_per_repo: HashMap<String, u32>, // Her repo için PR sayısı
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct GithubRepo {
    pub repo_url: String, 
    pub total_pull_requests: u32,
    pub pull_request_limit: u32, 
    pub reward_per_pull_request: u64, // Her pull request için ödül miktarı 
    pub owner_wallet_address: [u8; 32],  // repo sahibinin cuzdan adresi
    pub pull_request_contributors: HashMap<String, u32>, // Her kullanıcı için PR sayısı
}


impl Default for User {
    fn default() -> Self {
        User {
            github_username: String::new(),
            phantom_wallet: [0; 32],
            total_pull_request_count: 0,
            pull_requests_per_repo: HashMap::new(),
        }
    }
}


// twitter

