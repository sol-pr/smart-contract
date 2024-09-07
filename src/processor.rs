
use core::borrow;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{ 
    account_info::{next_account_info, AccountInfo},entrypoint::ProgramResult, lamports, msg, program::{invoke, invoke_signed}, program_error::ProgramError, pubkey::{self, Pubkey}, rent::Rent, system_instruction::{self}, system_program, sysvar::Sysvar
    };
    use crate::{instruction::RNGProgramInstruction, state::{GithubRepo, User, PrCount}};
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
          RNGProgramInstruction::CreateUser { github_username, phantom_wallet } => {
            // create_user fonksiyonunu çağır
            Self::create_user(accounts, _program_id, github_username, phantom_wallet)
          },
          RNGProgramInstruction:: PullRequestCount => {
            Self::pull_request_count( accounts,_program_id)
             },
          RNGProgramInstruction:: CreateRepo{GithubRepo}  => {
            Self::create_repo( accounts,_program_id,GithubRepo)
             },
          RNGProgramInstruction:: Transfer => {
            Self::transfer_reward( accounts,_program_id)
            },
          RNGProgramInstruction:: GetUser{phantom_wallet}  => {
            Self::get_user( accounts,_program_id,phantom_wallet)
              },
          RNGProgramInstruction:: GetPRepo => {
             Self::get_pull_requests_per_repo( accounts,_program_id)
             },
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
          return Err(ProgramError::MissingRequiredSignature);
        }
         // PDA hesabı oluşturma
       let (user_pda_address, bump) = Pubkey::find_program_address(
      &[b"user_pda", github_username.as_bytes()], 
      program_id
         );

         // 'user' hesabı PDA adresi ile aynı mı kontrol et
        if &user_pda_address != user.key {
      msg!("Provided user account does not match derived PDA.");
      return Err(ProgramError::InvalidArgument);
         }

         // hesabin bos olup olmadigini kontrol ederek hesabin olup olmadigina bakariz
          if user.lamports() > 0 {
            msg!("User with this Pubkey already exists.");
            return Err(ProgramError::AccountAlreadyInitialized);
          }

          // hesap yoksa olustur
        let rent = Rent::default();
        let user_rent = rent.minimum_balance(52);

        invoke_signed ( 
          &system_instruction::create_account(payer.key, &user_pda_address, user_rent,52, program_id),
          &[user.clone(),payer.clone()],
          &[
            &[b"user_pda",github_username.as_bytes(), &[bump]]
          ]
          )?;
          
        let user_info = User {
          github_username,
          phantom_wallet,
         };
        
        user_info.serialize(&mut &mut user.try_borrow_mut_data()?[..])?;
        
        Ok(())
      }

      pub fn pull_request_count (
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user = next_account_info(account_info_iter)?;
  
        let user_data = User::try_from_slice(&user.data.borrow())?;

        let rent = Rent:: default();
        let pr_count_rent = rent.minimum_balance(8);
  
        let(pr_counter_address, bump) = Pubkey::find_program_address(
          &[b"pull request counter", &user_data.phantom_wallet], 
          program_id);
  
        invoke_signed ( 
          &system_instruction::create_account(payer.key, &pr_counter_address,pr_count_rent , 8, program_id),
          &[user.clone(),payer.clone()],
          &[
            &[b"pull request counter",  &user_data.phantom_wallet, &[bump]]
          ]
        )?;

        Ok(())
      }

      pub fn create_repo (
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
        let repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;
        
        let(repo_pda_address, bump) = Pubkey::find_program_address(
          &[b"repo_pda",repo_data.repo_url.as_bytes()], 
          program_id);

          let mut serialized_data = vec![];
          data.serialize(&mut serialized_data)?;

        let rent = Rent::default();
        let repo_rent = rent.minimum_balance(serialized_data.len());

        invoke_signed ( 
          &system_instruction::create_account(payer.key, &repo_pda_address, repo_rent, serialized_data.len() as u64, program_id),
          &[github_repo_account.clone(),payer.clone()],
          &[
            &[b"repo_pda",repo_data.repo_url.as_bytes(), &[bump]]
          ]
          )?;

        // Veriyi yeni hesaba kaydet
        let mut repo_data_account = github_repo_account.try_borrow_mut_data()?;

        // Serileştirilmiş veriyi doğrudan kaydedin
          repo_data_account[..serialized_data.len()].copy_from_slice(&serialized_data);
        
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
        let pr_count = next_account_info(account_info_iter)?;

        if !payer.is_signer {
          msg!("payer is not a signer");
          return Err(ProgramError::MissingRequiredSignature);
       }

        let mut user_data = User::try_from_slice(&user.data.borrow())?;
        let mut repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;
        let mut pr_count_data = PrCount::try_from_slice(&pr_count.data.borrow())?;


        let (transfer_pda_address, bump) = Pubkey::find_program_address(
          &[b"transfer",user_data.github_username.as_bytes(),repo_data.repo_url.as_bytes()], 
          program_id);

        // pr sayisi limitine ulasildi mi?
        if pr_count_data.prcount >= repo_data.pull_request_limit {

          let transfer_amount = repo_data.reward_per_pull_request;
          
          let user_wallet_address = Pubkey::new(&user_data.phantom_wallet);
  
           // Ödül transfer talimatı
           let transfer_instruction = system_instruction::transfer(
            payer.key,
            &user_wallet_address,
            transfer_amount,
           );
           
            // Ödül transferini gerçekleştir
           invoke(
            &transfer_instruction,
            &[payer.clone(), user.clone()],
        )?;

        // count guncelle
           pr_count_data.prcount = pr_count_data.prcount.checked_sub(repo_data.pull_request_limit).ok_or(ProgramError::InvalidAccountData)?;
        
      }
        // pr sayisini arttiralim
        pr_count_data.prcount = pr_count_data.prcount.checked_add(1).ok_or(InvalidInstruction)?;

        let mut pr_count_data_account = pr_count.try_borrow_mut_data()?;
        pr_count_data.serialize(&mut &mut pr_count_data_account[..])?;
    
         Ok(())
  }
 
 // parametre gelen publickey varsa getir
 pub fn get_user(
  accounts: &[AccountInfo],
 _program_id: &Pubkey,
  phantom_wallet: [u8; 32],
    ) -> ProgramResult {
 let account_info_iter = &mut accounts.iter();
 let user = next_account_info(account_info_iter)?;

    // verileri oku
    let user_data = User::try_from_slice(&user.data.borrow())?;
     
     // parametre geln phantom wallet adresi ile kullancinin adresi ayni mi?
    if user_data.phantom_wallet != phantom_wallet {
      msg!("No user found with the provided phantom wallet.");
      return Err(ProgramError::InvalidArgument);
    }

    msg!(
      "User: {}, Phantom Wallet: {:?}",
      user_data.github_username,
      user_data.phantom_wallet
  );
      
 Ok(())
}
    // Hangi repo kac pull request
      pub fn get_pull_requests_per_repo(
      accounts: &[AccountInfo],
     _program_id: &Pubkey,
        ) -> ProgramResult {
     let account_info_iter = &mut accounts.iter();
     let user = next_account_info(account_info_iter)?;
     let github_repo_account = next_account_info(account_info_iter)?;
     let pr_count = next_account_info(account_info_iter)?;

    // verileri oku
    let user_data = User::try_from_slice(&user.data.borrow())?;
    let repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;
    let pr_count_data = PrCount::try_from_slice(&pr_count.data.borrow())?;

          msg!(
            "User: {}, Repo: {}, Pull Requests: {}",
        user_data.github_username,
        repo_data.repo_url,
        pr_count_data.prcount
          );
     Ok(())
  }
 }

     
     