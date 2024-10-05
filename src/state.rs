use borsh_derive::{BorshDeserialize, BorshSerialize};

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
pub struct PrCountAccess {
    pub id: String,
    pub user_phantom_wallet: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct GithubRepo {
    pub id: String,
    pub repo_url: String,
    pub repo_name: String,
    pub repo_description: String,
    pub total_pull_requests: u64,
    pub pull_request_limit: u64,
    pub reward_per_pull_request: u64,
    pub owner_wallet_address: [u8; 32],
    pub repo_wallet_address: [u8; 32],
}
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct RepoWalletAccount {
    pub repo_wallet_address: [u8; 32],
    
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct PrCount {
    pub prcount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
pub struct LoudBountyAccount {
    pub amount: u64,
    
}
// #[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
// pub struct CheckTransfer {
//     pub phantom_wallet: [u8; 32],
//     pub id: String,
// }
