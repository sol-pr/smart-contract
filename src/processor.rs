
use core::borrow;

use ahash::{HashMap, HashMapExt};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{ 
    account_info::{next_account_info, AccountInfo}, clock, config, entrypoint::ProgramResult, lamports, msg, program::{invoke, invoke_signed}, program_error::ProgramError, pubkey::{self, Pubkey}, rent::Rent, system_instruction::{self}, system_program, sysvar::Sysvar
    };
    use crate::{instruction::RNGProgramInstruction, state::{GithubRepo, User}};
    use crate::error::RNGProgramError::{InvalidInstruction};
    pub struct Processor;
    impl Processor {
    pub fn process(
      _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
      ) -> ProgramResult {
        let instruction: RNGProgramInstruction = RNGProgramInstruction::unpack(instruction_data)?;
    
    
        match instruction { 
       
        
        }
      }

      pub fn create_user (
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        github_username: String,
        phantom_wallet: [u8; 32],
      ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user = next_account_info(account_info_iter)?;
        
        if !payer.is_signer{ 
          msg!("payer is not a signer");
          // return Err(AuthorityError.into());
        }
        
        let(user_pda_address, bump) = Pubkey::find_program_address(&[b"user_pda", payer.key.as_ref() ], program_id);

        // min rent bedeli icin user structini serilestiririz
        let user_info = User {
          github_username,
          phantom_wallet,
          total_pull_request_count:0,
          pull_requests_per_repo: HashMap::new(),
      };

        let mut serialized_data = vec![];
        user_info.serialize(&mut serialized_data)?;

        let rent = Rent::default();
        let user_rent = rent.minimum_balance(serialized_data.len());

        invoke_signed ( 
          &system_instruction::create_account(payer.key, &user_pda_address, user_rent,serialized_data.len() as u64, program_id),
          &[user.clone(),payer.clone()],
          &[
            &[b"user_pda",payer.key.as_ref(), &[bump]]
          ]
          )?;
    
        user_info.serialize(&mut &mut user.try_borrow_mut_data()?[..])?;
        
        Ok(())
      }

      pub fn github_repo (
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        data:GithubRepo,
      ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;
        
        if !payer.is_signer{ 
          msg!("payer is not a signer");
          // return Err(AuthorityError.into());
        }
        
        let(repo_pda_address, bump) = Pubkey::find_program_address(&[b"repo_pda", payer.key.as_ref() ], program_id);

        let mut github_repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;
       
        let github_repo = GithubRepo {
            repo_url:github_repo_data.repo_url,
            total_pull_requests:github_repo_data.total_pull_requests,
            pull_request_limit:github_repo_data.pull_request_limit,
            reward_per_pull_request:github_repo_data.reward_per_pull_request,
            owner_wallet_address:github_repo_data.owner_wallet_address,
            pull_request_contributors: HashMap::new(),
        };

         let mut serialized_data = vec![];
         github_repo.serialize(&mut serialized_data)?;

        let rent = Rent::default();
        let repo_rent = rent.minimum_balance(serialized_data.len());

        invoke_signed ( 
          &system_instruction::create_account(payer.key, &repo_pda_address, repo_rent, serialized_data.len() as u64, program_id),
          &[github_repo_account.clone(),payer.clone()],
          &[
            &[b"repo_pda", payer.key.as_ref(), &[bump]]
          ]
          )?;

          github_repo.serialize(&mut &mut github_repo_account.try_borrow_mut_data()?[..])?;
        
        Ok(())
      }
     
      // pull request yapma fonks
      pub fn make_pull_request (
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        repo_url: String,
      ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;

        if !payer.is_signer {
          msg!("payer is not a signer");
          return Err(ProgramError::MissingRequiredSignature);
      }
        let mut user_data = User::try_from_slice(&user.data.borrow())?;
        let mut repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;

         // Kullanıcının repo için pull request sayısını güncelleyelim
        let user_repo_pull_requests = user_data.pull_requests_per_repo.entry(repo_url.clone()).or_insert(0);
        *user_repo_pull_requests += 1;

        // toplam pr sayisini guncelleyelim
        user_data.total_pull_request_count = user_data.total_pull_request_count.checked_add(1).ok_or(InvalidInstruction)?;

        user_data.serialize(&mut &mut user.try_borrow_mut_data()?[..])?;
        repo_data.serialize(&mut &mut github_repo_account.try_borrow_mut_data()?[..])?;

        Ok(())
      }

      // odul transfer fonks
      pub fn transfer_reward (
        accounts: &[AccountInfo],
        program_id: &Pubkey,
      ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;

        if !payer.is_signer {
          msg!("payer is not a signer");
          return Err(ProgramError::MissingRequiredSignature);
      }
        let mut user_data = User::try_from_slice(&user.data.borrow())?;
        let mut repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;

      // Kullanıcının repo için pull request sayısını kontrol et
        if let Some(&user_repo_pull_requests) = user_data.pull_requests_per_repo.get(&repo_data.repo_url) {

        if user_repo_pull_requests >= repo_data.pull_request_limit {
          let transfer_amount = repo_data.reward_per_pull_request;

        let (user_pda_address, bump) = Pubkey::find_program_address(
          &[b"user_pda", payer.key.as_ref()],
          program_id,
         );

         let transfer_instruction = system_instruction::transfer(
          payer.key,
          &user_pda_address,
          transfer_amount,
         );

      invoke_signed(
          &transfer_instruction,
          &[payer.clone(), user.clone()],
          &[&[b"user_pda", payer.key.as_ref(), &[bump]]],
      )?;

       msg!("Reward transferred successfully.");
        } else {
       msg!("Pull request limit has not been reached.");
         }
          } else {
        msg!("No pull request data found for the given repo.");
         }

         Ok(())
}
 
    // Repo ve PR sayısını goruntule
      pub fn get_pull_requests_per_repo(
      accounts: &[AccountInfo],
     _program_id: &Pubkey,
        ) -> ProgramResult {
     let account_info_iter = &mut accounts.iter();
     let user = next_account_info(account_info_iter)?;

    // Kullanıcı verilerini oku
    let user_data = User::try_from_slice(&user.data.borrow())?;

    // Repo ve PR sayısını yazdır
   for (repo_url, pr_count) in user_data.pull_requests_per_repo {
      msg!("Repo: {}, Pull Requests: {}", repo_url, pr_count);
  }

  Ok(())
}
 }
       