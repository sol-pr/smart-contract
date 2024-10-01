use crate::{
    error::RNGProgramError::InvalidInstruction,
    state::{GithubRepo, PrCount, User, UserForCreate},
};
use borsh::BorshDeserialize;
use solana_program::{msg, program_error::ProgramError};

#[derive(Debug, PartialEq)]
pub enum RNGProgramInstruction {
    TotalPrCount {
        User: User,
    },
    PullRequestCount {
        PrCount: PrCount,
    },
    ManageUser {
        github_username: String,
        phantom_wallet: [u8; 32],
    },
    GetUser {
        phantom_wallet: [u8; 32],
    },
    CreateRepo {
        GithubRepo: GithubRepo,
    },
    GetRepo,
    GetRepoUrl {
        id: String,
    },
    Transfer,

    GetPRepo,
}

impl RNGProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::TotalPrCount {
                User: User::try_from_slice(&rest)?,
            },
            1 => Self::PullRequestCount {
                PrCount: PrCount::try_from_slice(&rest)?,
            },
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
                GithubRepo: GithubRepo::try_from_slice(&rest)?,
            },
            5 => Self::GetRepo,
            6 => {
                let repo: GithubRepo = GithubRepo::try_from_slice(&rest)?;
                Self::GetRepoUrl { id: repo.id }
            }
            7 => Self::Transfer,
            8 => Self::GetPRepo,
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
