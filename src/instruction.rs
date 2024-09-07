use crate::{error::RNGProgramError::InvalidInstruction, state::{GithubRepo, User,PrCount}, };
use borsh::BorshDeserialize;
use solana_program::{msg, program_error::ProgramError};

#[derive(Debug, PartialEq)]
pub enum RNGProgramInstruction { 
  CreateUser { github_username: String, phantom_wallet: [u8; 32] },
  PullRequestCount,
  CreateRepo{GithubRepo:GithubRepo},
  Transfer,
  GetUser{phantom_wallet:[u8; 32]},
  GetPRepo,
}
  

impl RNGProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
      let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
       
      Ok(match tag {
        0 => {
          let user: User = User::try_from_slice(&rest)?;
          Self::CreateUser {
              github_username: user.github_username,
              phantom_wallet: user.phantom_wallet,
          }
        },
        1 => Self::PullRequestCount,
        2=> Self::CreateRepo{
          GithubRepo:GithubRepo::try_from_slice(&rest)?
        },
        3 => Self::Transfer,
        4=> {
          let user: User = User::try_from_slice(&rest)?;
          Self::GetUser {
              phantom_wallet:user.phantom_wallet,
          }
        },
        5 => Self::GetPRepo,
        _ => return Err(InvalidInstruction.into()),
      })
    }
  }
  
  