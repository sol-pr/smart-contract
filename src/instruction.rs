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
    ManageUser {
        github_username: String,
        phantom_wallet: [u8; 32],
    },
    PullRequestCount {
        PrCount: PrCount,
    },
    CreateRepo {
        GithubRepo: GithubRepo,
    },
    Transfer,
    GetUser {
        phantom_wallet: [u8; 32],
    },
    GetPRepo,
}

impl RNGProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        /*
          4 => {
            let drawdata = DrawData::try_from_slice(&rest).map_err(||InvalidInstruction)?;
            Self::Draw {
                prize_amount: draw_data.prize_amount,
                winning_numbers: draw_data.winning_numbers,
            }
        }

           */
        Ok(match tag {
            0 => Self::TotalPrCount {
                User: User::try_from_slice(&rest)?,
            },
            1 => {
                let user = UserForCreate::try_from_slice(&rest).map_err(|_| InvalidInstruction)?;
                Self::ManageUser {
                    github_username: user.github_username,
                    phantom_wallet: user.phantom_wallet,
                }
            }
            2 => Self::PullRequestCount {
                PrCount: PrCount::try_from_slice(&rest)?,
            },
            3 => Self::CreateRepo {
                GithubRepo: GithubRepo::try_from_slice(&rest)?,
            },
            4 => Self::Transfer,
            5 => {
                let user: User = User::try_from_slice(&rest)?;
                Self::GetUser {
                    phantom_wallet: user.phantom_wallet,
                }
            }
            6 => Self::GetPRepo,
            _ => return Err(InvalidInstruction.into()),
        })
    }
}
