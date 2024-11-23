use crate::{
    error::RNGProgramError::InvalidInstruction,
    state::{GithubRepo, LoadBounty, PrCountAccess, UserForCreate},
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
    CreateRepo {
        github_repo: GithubRepo,
    },
    Transfer,
    LoadBounty {
        data: LoadBounty,
    },
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
                let github_repo =
                    GithubRepo::try_from_slice(&rest).map_err(|_| InvalidInstruction)?;
                Self::CreateRepo { github_repo }
            }
            4 => Self::Transfer,
            5 => {
                let data: LoadBounty =
                    LoadBounty::try_from_slice(&rest).map_err(|_| InvalidInstruction)?;
                Self::LoadBounty { data }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
