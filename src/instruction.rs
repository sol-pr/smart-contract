use crate::{error::RNGProgramError::InvalidInstruction, state::{}, };
use borsh::BorshDeserialize;
use solana_program::{msg, program_error::ProgramError};

#[derive(Debug, PartialEq)]
pub enum RNGProgramInstruction { 
 
  
}

impl RNGProgramInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
  
      let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
       
      Ok(match tag {
        
        _ => return Err(InvalidInstruction.into()),
      })
    }
  }
  
  