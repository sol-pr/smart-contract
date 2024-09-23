use core::borrow;

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
            RNGProgramInstruction::ManageUser { User } => {
                Self::create_user(accounts, _program_id, User)
            }
            RNGProgramInstruction::PullRequestCount { PrCount } => {
                Self::pull_request_count(accounts, _program_id, PrCount)
            }
            RNGProgramInstruction::CreateRepo { GithubRepo } => {
                Self::create_repo(accounts, _program_id, GithubRepo)
            }
            RNGProgramInstruction::Transfer => Self::transfer_reward(accounts, _program_id),
            RNGProgramInstruction::GetUser { phantom_wallet } => {
                Self::get_user(accounts, _program_id, phantom_wallet)
            }
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

    // kullanci kontrolu, yoksa olustur
    pub fn create_user(accounts: &[AccountInfo], program_id: &Pubkey, data: User) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user = next_account_info(account_info_iter)?;

        if !payer.is_signer {
            msg!("payer is not a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        // PDA hesabı oluşturma
        let (user_pda_address, bump) =
            Pubkey::find_program_address(&[b"user_pda", &data.phantom_wallet], program_id);

        // 'user' hesabı PDA adresi ile aynı mı kontrol et
        if &user_pda_address != user.key {
            msg!("Provided user account does not match derived PDA.");
            return Err(ProgramError::InvalidArgument);
        }

        let mut serialized_data = vec![];
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

        // Yeni kullanıcı bilgilerini kaydet
        let user_info = User {
            github_username: data.github_username,
            phantom_wallet: data.phantom_wallet,
            totalearn: 0,
            submitted_at: 0,
            total_pr_count: 0,
        };

        user_info.serialize(&mut &mut user.try_borrow_mut_data()?[..])?;

        msg!("New user created and stored.");

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

        let mut serialized_data = vec![];
        data.serialize(&mut serialized_data)?;

        let rent = Rent::default();
        let pr_count_rent = rent.minimum_balance(serialized_data.len());

        let (pr_counter_address, bump) =
            Pubkey::find_program_address(&[b"pull request counter"], program_id);

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                &pr_counter_address,
                pr_count_rent,
                serialized_data.len() as u64,
                program_id,
            ),
            &[payer.clone()],
            &[&[b"pull request counter", &[bump]]],
        )?;

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

        if !payer.is_signer {
            msg!("payer is not a signer");
            // return Err(AuthorityError.into());
        }

        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp as u64;

        let repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;

        let (repo_pda_address, bump) =
            Pubkey::find_program_address(&[b"repo_pda", repo_data.repo_url.as_bytes()], program_id);

        let mut serialized_data = vec![];
        data.serialize(&mut serialized_data)?;

        let rent = Rent::default();
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
            &[&[b"repo_pda", repo_data.repo_url.as_bytes(), &[bump]]],
        )?;

        // Yeni repo verisini oluşturun
        let repo_info = GithubRepo {
            repo_url: data.repo_url,
            repo_name: data.repo_name,
            repo_description: data.repo_description,
            total_pull_requests: 0,
            pull_request_limit: data.pull_request_limit,
            reward_per_pull_request: data.reward_per_pull_request,
            owner_wallet_address: data.owner_wallet_address, // repo saihibinin cuzdan adresi
            created_at: current_time,                        //mevcut zaman
        };

        repo_info.serialize(&mut &mut github_repo_account.try_borrow_mut_data()?[..])?;

        Ok(())
    }

    // halihazirda olan repolari goruntule
    pub fn get_repos(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        filter_date: u64, // bu tarihten sonra olusturulmus repolari goruntule
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;

        if !payer.is_signer {
            msg!("payer is not a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;

        // Eğer repo 'filter_date' den daha yeni oluşturulmuşsa göster
        if repo_data.created_at > filter_date {
            msg!("Newest repo:");
            msg!("Repo URL: {}", repo_data.repo_url);
            msg!("Repo Name: {}", repo_data.repo_name);
            msg!("Repo Description: {}", repo_data.repo_description);
            msg!("Creation Date: {}", repo_data.created_at);
        } else {
            msg!("No new repo was found after the specified date.");
        }

        Ok(())
    }

    // odul transfer fonks
    pub fn transfer_reward(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let user = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;
        let pr_count = next_account_info(account_info_iter)?;
        let total_pr_count = next_account_info(account_info_iter)?;

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

    // parametre gelen publickey varsa getir
    pub fn get_user(
        accounts: &[AccountInfo],
        _program_id: &Pubkey,
        phantom_wallet: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;

        // verileri oku
        let mut user_data = User::try_from_slice(&user.data.borrow())?;

        // parametre geln phantom wallet adresi ile kullancinin adresi ayni mi?
        if user_data.phantom_wallet != phantom_wallet {
            msg!("No user found with the provided phantom wallet.");
            return Err(ProgramError::InvalidArgument);
        }

        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp as u64;

        // 1 hafta 604800 saniye
        let one_week_in_seconds = 604800;

        // Eğer bir hafta geçmişse haftalik pr sayisini sifirlariz
        if current_time - user_data.submitted_at >= one_week_in_seconds {
            msg!("A week has passed since the last PR count reset. Resetting weekly PR count.");
            user_data.total_pr_count = 0;
            user_data.submitted_at = current_time;
        }
        msg!(
            "User: {}, Phantom Wallet: {:?}, Weekly PR Count: {}, Total Earnings: {}",
            user_data.github_username,
            user_data.phantom_wallet,
            user_data.total_pr_count,
            user_data.totalearn
        );

        user_data.serialize(&mut &mut user.try_borrow_mut_data()?[..])?;

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

//  4-En yeni repolar getirilecek, bunun için repo struct içine oluşturma tarihi ekleencek
