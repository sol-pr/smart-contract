use crate::{
    error::RNGProgramError::InvalidInstruction,
    state::{GithubRepo, PrCountAccess, User, UserForCreate},
};
use borsh::BorshDeserialize;
use solana_program::program_error::ProgramError;

#[derive(Debug, PartialEq)]
pub enum RNGProgramInstruction {
    CreatePrCount {
        id: String,
        user_phantom_wallet: [u8; 32],
    },
    IncreaseRequestCount {
        id: String,
        user_phantom_wallet: [u8; 32],
    },
    ManageUser {
        github_username: String,
        phantom_wallet: [u8; 32],
    },
    GetUser {
        phantom_wallet: [u8; 32],
    },
    CreateRepo {
        github_repo: GithubRepo,
    },
    GetRepo,
    GetRepoUrl {
        id: String,
    },
    Transfer,
    LoasBountyRepo {
        amount: u64,
    },
    GetPRepo,
}

impl RNGProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let pr = PrCountAccess::try_from_slice(&rest).map_err(|_| InvalidInstruction)?;
                Self::CreatePrCount {
                    id: pr.id,
                    user_phantom_wallet: pr.user_phantom_wallet,
                }
            }
            1 => {
                let pr = PrCountAccess::try_from_slice(&rest).map_err(|_| InvalidInstruction)?;
                Self::IncreaseRequestCount {
                    id: pr.id,
                    user_phantom_wallet: pr.user_phantom_wallet,
                }
            }
            2 => {
                let user = UserForCreate::try_from_slice(&rest).map_err(|_| InvalidInstruction)?;
                Self::ManageUser {
                    github_username: user.github_username,
                    phantom_wallet: user.phantom_wallet,
                }
            }
            3 => {
                let user: User = User::try_from_slice(&rest)?;
                Self::GetUser {
                    phantom_wallet: user.phantom_wallet,
                }
            }
            4 => Self::CreateRepo {
                github_repo: GithubRepo::try_from_slice(&rest)?,
            },
            5 => Self::GetRepo,
            6 => {
                let repo: GithubRepo = GithubRepo::try_from_slice(&rest)?;
                Self::GetRepoUrl { id: repo.id }
            }
            7 => Self::Transfer,
            8 => Self::LoasBountyRepo {
                amount: u64::try_from_slice(&rest)?,
            },
            9 => Self::GetPRepo,
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
