use crate::{error::RNGProgramError::InvalidInstruction, state::{GithubRepo, User,PrCount}, };
use borsh::BorshDeserialize;
use solana_program::{msg, program_error::ProgramError};

#[derive(Debug, PartialEq)]
pub enum RNGProgramInstruction { 
  TotalPrCount{User:User},
  ManageUser{User:User},
  PullRequestCount{PrCount:PrCount},
  CreateRepo{GithubRepo:GithubRepo},
  Transfer,
  GetUser{phantom_wallet:[u8; 32]},
  GetPRepo,
}
  

impl RNGProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
      let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
       
      Ok(match tag {
        0=> Self::TotalPrCount{
          User:User::try_from_slice(&rest)?
        },
        1=> Self::ManageUser{
          User:User::try_from_slice(&rest)?
        },
        2 => Self::PullRequestCount{
          PrCount:PrCount::try_from_slice(&rest)?
        },
        3=> Self::CreateRepo{
          GithubRepo:GithubRepo::try_from_slice(&rest)?
        },
        4 => Self::Transfer,
        5=> {
          let user: User = User::try_from_slice(&rest)?;
          Self::GetUser {
              phantom_wallet:user.phantom_wallet,
          }
        },
        6 => Self::GetPRepo,
        _ => return Err(InvalidInstruction.into()),
      })
    }
  }
  
  