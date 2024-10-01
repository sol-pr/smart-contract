use core::borrow;
use std::net::ToSocketAddrs;

use crate::error::RNGProgramError::InvalidInstruction;
use crate::{
    instruction::RNGProgramInstruction,
    state::{GithubRepo, PrCount, User},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::{self, Clock},
    entrypoint::ProgramResult,
    lamports, msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::{self, Pubkey},
    rent::Rent,
    system_instruction::{self},
    system_program,
    sysvar::Sysvar,
};
pub struct Processor;
impl Processor {
    pub fn process(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction: RNGProgramInstruction = RNGProgramInstruction::unpack(instruction_data)?;

        match instruction {
            RNGProgramInstruction::TotalPrCount { User } => {
                Self::total_pull_request_count(accounts, _program_id, User)
            }

            RNGProgramInstruction::PullRequestCount { PrCount } => {
                Self::pull_request_count(accounts, _program_id, PrCount)
            }

            RNGProgramInstruction::ManageUser {
                github_username,
                phantom_wallet,
            } => Self::create_user(accounts, _program_id, github_username, phantom_wallet),

            RNGProgramInstruction::GetUser { phantom_wallet } => {
                Self::get_user(accounts, _program_id, phantom_wallet)
            }

            RNGProgramInstruction::CreateRepo { GithubRepo } => {
                Self::create_repo(accounts, _program_id, GithubRepo)
            }

            RNGProgramInstruction::GetRepo { GithubRepo } => {
                Self::get_repos(accounts, _program_id, GithubRepo)
            }

            RNGProgramInstruction::GetRepoUrl { id } => {
                Self::get_repo_by_id(accounts, _program_id, id)
            }

            RNGProgramInstruction::Transfer => Self::transfer_reward(accounts, _program_id),

            RNGProgramInstruction::GetPRepo => {
                Self::get_pull_requests_per_repo(accounts, _program_id)
            }
        }
    }

    // Kullanicinin yaptigi toplam pr sayisini sayan sayac
    pub fn total_pull_request_count(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        data: User,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user = next_account_info(account_info_iter)?;

        let user_data = User::try_from_slice(&user.data.borrow())?;

        let mut serialized_data = vec![];
        data.serialize(&mut serialized_data)?;

        let rent = Rent::default();
        let total_pr_count_rent = rent.minimum_balance(serialized_data.len());

        let (total_pr_counter_address, bump) = Pubkey::find_program_address(
            &[
                b"total pull request counter",
                &user_data.phantom_wallet,
                user_data.github_username.to_string().as_ref(),
            ],
            program_id,
        );

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                &total_pr_counter_address,
                total_pr_count_rent,
                serialized_data.len() as u64,
                program_id,
            ),
            &[user.clone(), payer.clone()],
            &[&[
                b"total pull request counter",
                &user_data.phantom_wallet,
                user_data.github_username.to_string().as_ref(),
                &[bump],
            ]],
        )?;

        Ok(())
    }

    // odul icin repo basina pr sayisini sayan sayac
    pub fn pull_request_count(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        data: PrCount,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;

        let user_data = User::try_from_slice(&user.data.borrow())?;
        let repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;

        let mut serialized_data = vec![];
        data.serialize(&mut serialized_data)?;

        let rent = Rent::default();
        let pr_count_rent = rent.minimum_balance(serialized_data.len());

        let (pr_counter_address, bump) = Pubkey::find_program_address(
            &[
                b"pull request counter",
                user_data.github_username.to_string().as_ref(),
                repo_data.repo_url.to_string().as_ref(),
            ],
            program_id,
        );

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                &pr_counter_address,
                pr_count_rent,
                serialized_data.len() as u64,
                program_id,
            ),
            &[payer.clone()],
            &[&[
                b"pull request counter",
                user_data.github_username.to_string().as_ref(),
                repo_data.repo_url.to_string().as_ref(),
                &[bump],
            ]],
        )?;

        Ok(())
    }

    pub fn get_pull_request_count(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;

        let user_data = User::try_from_slice(&user.data.borrow())?;
        let repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;

        let (pr_counter_address, _bump) = Pubkey::find_program_address(
            &[
                b"pull request counter",
                user_data.github_username.to_string().as_ref(),
                repo_data.repo_url.to_string().as_ref(),
            ],
            program_id,
        );

        let pr_counter_account = next_account_info(account_info_iter)?;

        if pr_counter_account.key != &pr_counter_address {
            return Err(ProgramError::InvalidAccountData);
        }

        let pr_count_data = PrCount::try_from_slice(&pr_counter_account.data.borrow())?;

        msg!("Pull Request count: {}", pr_count_data.prcount);

        Ok(())
    }

    // kullanci kontrolu, yoksa olustur
    pub fn create_user(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        github_user_name: String,
        user_phantom_wallet: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user = next_account_info(account_info_iter)?;

        if !payer.is_signer {
            msg!("payer is not a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // PDA hesabı oluşturma
        let (user_pda_address, bump) =
            Pubkey::find_program_address(&[b"user_pda", &user_phantom_wallet], program_id);

        // 'user' hesabı PDA adresi ile aynı mı kontrol et
        if user_pda_address != *user.key {
            msg!("Provided user account does not match derived PDA.");
            return Err(ProgramError::InvalidArgument);
        }

        let mut serialized_data = vec![];

        let data = User {
            github_username: github_user_name,
            phantom_wallet: user_phantom_wallet,
            totalearn: 0,
            submitted_at: 0,
            total_pr_count: 0,
        };
        data.serialize(&mut serialized_data)?;

        let rent = Rent::default();
        let user_rent = rent.minimum_balance(serialized_data.len());

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                &user_pda_address,
                user_rent,
                serialized_data.len() as u64,
                program_id,
            ),
            &[user.clone(), payer.clone()],
            &[&[b"user_pda", &data.phantom_wallet, &[bump]]],
        )?;

        // Kullanıcı bilgilerini kaydet
        data.serialize(&mut &mut user.try_borrow_mut_data()?[..])?;

        msg!("New user created and stored.");

        Ok(())
    }

    // parametre gelen publickey varsa getir
    pub fn get_user(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        phantom_wallet: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_account = next_account_info(account_info_iter)?;

        // Kullanıcı hesabının PDA adresi ile eşleşip eşleşmediğini kontrol et
        let user_data = User::try_from_slice(&user_account.data.borrow())?;

        let (user_pda_address, bump) =
            Pubkey::find_program_address(&[b"user_pda", &phantom_wallet], program_id);

        let phantom_wallet_pubkey = Pubkey::new_from_array(phantom_wallet);

        if user_pda_address != phantom_wallet_pubkey {
            msg!("Provided public key does not match the derived PDA.");
            return Err(ProgramError::InvalidArgument);
        }

        msg!(
            "User: {}, Phantom Wallet: {:?}, Weekly PR Count: {}, Total Earnings: {}",
            user_data.github_username,
            user_data.phantom_wallet,
            user_data.total_pr_count,
            user_data.totalearn
        );

        user_data.serialize(&mut &mut user_account.try_borrow_mut_data()?[..])?;

        Ok(())
    }

    // yeni repo olustur
    pub fn create_repo(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        data: GithubRepo,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;

        // Payer'ın imzacı mi
        if !payer.is_signer {
            msg!("Payer is not a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Github repo account'un yazılabilir olduğundan emin ol
        if !github_repo_account.is_writable {
            msg!("GitHub repo account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }

        // PDA oluşturuluyor
        let (repo_pda_address, bump) =
            Pubkey::find_program_address(&[b"repo_pda", data.id.as_bytes()], program_id);

        // Veri yapılandırması (GitHub repo verisi)
        let repo_info = GithubRepo {
            id: data.id.clone(),
            repo_url: data.repo_url.clone(),
            repo_name: data.repo_name.clone(),
            repo_description: data.repo_description.clone(),
            total_pull_requests: 0,
            pull_request_limit: data.pull_request_limit,
            reward_per_pull_request: data.reward_per_pull_request,
            owner_wallet_address: data.owner_wallet_address,
        };

        // Serialize etmeden önce veriyi byte dizisine dönüştür
        let mut serialized_data = vec![];
        repo_info.serialize(&mut serialized_data)?;

        let rent = Rent::get()?;
        let repo_rent = rent.minimum_balance(serialized_data.len());

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                &repo_pda_address,
                repo_rent,
                serialized_data.len() as u64,
                program_id,
            ),
            &[github_repo_account.clone(), payer.clone()],
            &[&[b"repo_pda", data.id.as_bytes(), &[bump]]],
        )?;

        // Veriyi hesaba yazmadan önce alanın yeterli olup olmadığını kontrol et
        let github_repo_data_len = github_repo_account.try_data_len()?;
        if github_repo_data_len < serialized_data.len() {
            return Err(ProgramError::AccountDataTooSmall);
        }

        // Veriyi GitHub repo account'una yaz
        repo_info.serialize(&mut &mut github_repo_account.try_borrow_mut_data()?[..])?;

        Ok(())
    }

    // halihazirda olan repolari goruntule
    pub fn get_repos(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        data: GithubRepo,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let github_repo_account = next_account_info(account_info_iter)?;

        // Hesap verisini deserialize edin
        let repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;

        // PDA adresini kontrol et
        let (repo_pda_address, bump) =
            Pubkey::find_program_address(&[b"repo_pda", data.id.as_bytes()], program_id);

        if repo_pda_address != *github_repo_account.key {
            msg!("Provided public key does not match the derived PDA.");
            return Err(ProgramError::InvalidArgument);
        }

        // Repo bilgilerini mesaj olarak görüntüle
        msg!(
        "Github Repo Name: {}, Github Repo Url: {}, Github Repo Description: {}, Total Pull Requests: {}, 
        Pull Request Limit: {}, Reward Per Pull Request: {}, Owner Wallet Address: {:?}",
        repo_data.repo_name,
        repo_data.repo_url,
        repo_data.repo_description,
        repo_data.total_pull_requests,
        repo_data.pull_request_limit,
        repo_data.reward_per_pull_request,
        repo_data.owner_wallet_address,
    );

        repo_data.serialize(&mut &mut github_repo_account.try_borrow_mut_data()?[..])?;

        Ok(())
    }

    // repo url'e gore repoyu getir
    pub fn get_repo_by_id(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        id: String,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let github_repo_account = next_account_info(account_info_iter)?;

        let repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;

        let (repo_pda_address, _bump) =
            Pubkey::find_program_address(&[b"repo_pda", id.as_bytes()], program_id);

        if repo_pda_address != *github_repo_account.key {
            msg!(
                "Provided public key ({:?}) does not match the derived PDA ({:?}).",
                github_repo_account.key,
                repo_pda_address
            );
            return Err(ProgramError::InvalidArgument);
        }

        msg!(
        "Github Repo Name: {}, Github Repo Url: {}, Github Repo Description: {}, Total Pull Requests: {}, 
        Pull Request Limit: {}, Reward Per Pull Request: {}, Owner Wallet Address: {:?}",
        repo_data.repo_name,
        repo_data.repo_url,
        repo_data.repo_description,
        repo_data.total_pull_requests,
        repo_data.pull_request_limit,
        repo_data.reward_per_pull_request,
        repo_data.owner_wallet_address,
    );

        repo_data.serialize(&mut &mut github_repo_account.try_borrow_mut_data()?[..])?;

        Ok(())
    }

    // odul transfer fonks
    pub fn transfer_reward(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;
        let pr_count = next_account_info(account_info_iter)?;
        let _total_pr_count = next_account_info(account_info_iter)?;

        //Criteria chcek
        if !payer.is_signer {
            msg!("payer is not a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut user_data = User::try_from_slice(&user.data.borrow())?;
        let mut repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;
        let mut pr_count_data = PrCount::try_from_slice(&pr_count.data.borrow())?;

        let (transfer_pda_address, bump) = Pubkey::find_program_address(
            &[
                b"transfer",
                user_data.github_username.as_bytes(),
                repo_data.repo_url.as_bytes(),
            ],
            program_id,
        );

        // pr sayisi limitine ulasildi mi?
        if pr_count_data.prcount >= repo_data.pull_request_limit {
            let transfer_amount = repo_data.reward_per_pull_request;

            let user_wallet_address = Pubkey::new(&user_data.phantom_wallet);

            // Ödül transfer talimatı
            let transfer_instruction =
                system_instruction::transfer(payer.key, &user_wallet_address, transfer_amount);

            // Ödül transferini gerçekleştir
            invoke(&transfer_instruction, &[payer.clone(), user.clone()])?;

            // count guncelle
            pr_count_data.prcount = pr_count_data
                .prcount
                .checked_sub(repo_data.pull_request_limit)
                .ok_or(ProgramError::InvalidAccountData)?;
        }

        // pr sayisini arttiralim
        pr_count_data.prcount = pr_count_data
            .prcount
            .checked_add(1)
            .ok_or(InvalidInstruction)?;

        //top pr sayisini her kosulda arttircaz
        user_data.total_pr_count = user_data
            .total_pr_count
            .checked_add(1)
            .ok_or(ProgramError::InvalidAccountData)?;

        // toplam elde edolen kazanc
        user_data.totalearn = user_data
            .totalearn
            .checked_add(repo_data.reward_per_pull_request)
            .ok_or(InvalidInstruction)?;

        let mut pr_count_data_account = pr_count.try_borrow_mut_data()?;
        pr_count_data.serialize(&mut &mut pr_count_data_account[..])?;

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
