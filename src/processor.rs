use crate::error::RNGProgramError::ArithmeticErr;
use crate::{
    instruction::RNGProgramInstruction,
    state::{GithubRepo, LoudBountyAccount, PrCount, User},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::{self},
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
            RNGProgramInstruction::CreatePrCount {
                id,
                user_phantom_wallet,
            } => Self::create_pr_count(accounts, _program_id, id, user_phantom_wallet),

            RNGProgramInstruction::IncreaseRequestCount {
                id,
                user_phantom_wallet,
            } => Self::increase_pr_count(accounts, _program_id, id, user_phantom_wallet),

            RNGProgramInstruction::ManageUser {
                github_username,
                phantom_wallet,
            } => Self::create_user(accounts, _program_id, github_username, phantom_wallet),

            RNGProgramInstruction::GetUser { phantom_wallet } => {
                Self::get_user(accounts, _program_id, phantom_wallet)
            }

            RNGProgramInstruction::CreateRepo { github_repo } => {
                Self::create_repo(accounts, _program_id, github_repo)
            }

            RNGProgramInstruction::GetRepo => {
                let program_id = &_program_id; // Referansı alın
                Self::get_all_repos(accounts, program_id) // Uygun şekilde fonksiyonu çağırın
            }

            RNGProgramInstruction::GetRepoUrl { id } => {
                Self::get_repo_by_id(accounts, _program_id, id)
            }

            RNGProgramInstruction::Transfer => Self::transfer_reward(accounts, _program_id),

            RNGProgramInstruction::LoasBountyRepo { data } => {
                Self::load_bounty_repo(accounts, _program_id, data)
            }

            RNGProgramInstruction::GetPRepo => {
                Self::get_pull_requests_per_user(accounts, _program_id)
            }
        }
    }

    /// Create a new PR count account for a user and repo
    pub fn create_pr_count(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        id: String,
        user_phantom_wallet: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let pr_counter = next_account_info(account_info_iter)?;

        if !payer.is_signer {
            msg!("payer is not a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (pr_counter_pda, bump) = Pubkey::find_program_address(
            &[
                b"pull request counter",
                &user_phantom_wallet,
                &id.as_bytes(),
            ],
            &program_id,
        );

        // 'pda_counter' hesabı PDA adresi ile aynı mı kontrol et
        if pr_counter_pda != *pr_counter.key {
            msg!("Provided pda_counter account does not match derived PDA.");
            return Err(ProgramError::InvalidArgument);
        }

        let mut serialized_data = vec![];

        let data = PrCount { prcount: 1 };
        data.serialize(&mut serialized_data)?;

        let rent = Rent::default();
        let pr_count_rent = rent.minimum_balance(serialized_data.len());

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                &pr_counter_pda,
                pr_count_rent,
                serialized_data.len() as u64,
                &program_id,
            ),
            &[pr_counter.clone(), payer.clone()],
            &[&[
                b"pull request counter",
                &user_phantom_wallet,
                &id.as_bytes(),
                &[bump],
            ]],
        )?;

        data.serialize(&mut &mut pr_counter.try_borrow_mut_data()?[..])?;

        Ok(())
    }

    /// Increase the PR count for a user and repo
    pub fn increase_pr_count(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        id: String,
        user_phantom_wallet: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let pr_counter = next_account_info(account_info_iter)?;

        if !payer.is_signer {
            msg!("payer is not a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let (pr_counter_pda, _bump) = Pubkey::find_program_address(
            &[
                b"pull request counter",
                &user_phantom_wallet,
                &id.as_bytes(),
            ],
            &program_id,
        );

        // 'pda_counter' hesabı PDA adresi ile aynı mı kontrol et
        if pr_counter_pda != *pr_counter.key {
            msg!("Provided pda_counter account does not match derived PDA.");
            return Err(ProgramError::InvalidArgument);
        }

        let mut pr_counter_data = PrCount::try_from_slice(&pr_counter.data.borrow())?;

        pr_counter_data.prcount = pr_counter_data
            .prcount
            .checked_add(1)
            .ok_or(ArithmeticErr)?;

        pr_counter_data.serialize(&mut &mut pr_counter.try_borrow_mut_data()?[..])?;

        Ok(())
    }

    /// Create a new user account
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

    /// Get user account data
    pub fn get_user(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        phantom_wallet: [u8; 32],
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_account = next_account_info(account_info_iter)?;

        // Kullanıcı hesabının PDA adresi ile eşleşip eşleşmediğini kontrol et
        let user_data = User::try_from_slice(&user_account.data.borrow())?;

        let (user_pda_address, _bump) =
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

    /// Create a new GitHub repo account
    pub fn create_repo(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        data: GithubRepo,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;
        let repo_wallet_account = next_account_info(account_info_iter)?;

        let (github_repo_pda_address, bump) =
            Pubkey::find_program_address(&[b"repo_pda", data.id.as_bytes()], program_id);

        if github_repo_pda_address != *github_repo_account.key {
            msg!("Provided GitHub repo account does not match derived PDA.");
            return Err(ProgramError::InvalidArgument);
        }

        // Yeni bir cüzdan account'u oluşturuluyor (repo_wallet_address)
        let (repo_wallet_pda, wallet_bump) =
            Pubkey::find_program_address(&[b"repo_wallet", data.id.as_bytes()], program_id);

        let rent = Rent::default();
        let wallet_rent = rent.minimum_balance(0);

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                &repo_wallet_pda,
                wallet_rent,
                0,
                program_id,
            ),
            &[payer.clone(), repo_wallet_account.clone()],
            &[&[b"repo_wallet", data.id.as_bytes(), &[wallet_bump]]],
        )?;

        // Veri yapılandırması
        let repo_info = GithubRepo {
            id: data.id.clone(),
            repo_url: data.repo_url,
            repo_name: data.repo_name,
            repo_description: data.repo_description,
            total_pull_requests: 0,
            pull_request_limit: data.pull_request_limit,
            reward_per_pull_request: data.reward_per_pull_request,
            owner_wallet_address: data.owner_wallet_address,
            repo_wallet_address: repo_wallet_pda.to_bytes(),
        };

        let rent = Rent::default();
        let repo_rent = rent.minimum_balance(184);

        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                &github_repo_pda_address,
                repo_rent,
                184,
                program_id,
            ),
            &[github_repo_account.clone(), payer.clone()],
            &[&[b"repo_pda", data.id.clone().as_bytes(), &[bump]]],
        )?;

        // Veriyi GitHub repo account'una yaz
        repo_info.serialize(&mut &mut github_repo_account.try_borrow_mut_data()?[..])?;

        Ok(())
    }

    /// Get all GitHub repo accounts
    pub fn get_all_repos(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let mut github_repos: Vec<GithubRepo> = Vec::new();
        let account_info_iter = &mut accounts.iter();

        // Tüm hesapları dolaşarak GitHubRepo verilerini topluyoruz
        for account_info in account_info_iter {
            // PDA adresi olup olmadığını kontrol et
            if account_info.owner != program_id {
                msg!("Account does not belong to this program");
                continue;
            }

            // PDA adresini doğrulamak için yeniden hesapla
            let (expected_pda, _bump) = Pubkey::find_program_address(&[b"repo_pda"], program_id);
            if *account_info.key != expected_pda {
                msg!("Account is not the expected repo PDA");
                continue;
            }

            // Hesaptan gelen veri boyutunu alalım
            let data_len = account_info.try_data_len()?;

            // Eğer veri varsa, deserialize edip listemize ekleyelim
            if data_len > 0 {
                let repo_info = GithubRepo::try_from_slice(&account_info.try_borrow_data()?)?;
                github_repos.push(repo_info);
            }
        }

        // Tüm repoları serialize et
        let mut serialized_repos: Vec<u8> = Vec::new();
        github_repos
            .serialize(&mut serialized_repos)
            .map_err(|err| {
                msg!("Error serializing repo data: {:?}", err);
                ProgramError::InvalidAccountData
            })?;

        // Serialize edilen veriyi client'a gönder
        msg!("Serialized all Github repos: {:?}", serialized_repos);

        Ok(())
    }

    /// Get a GitHub repo account by ID
    pub fn get_repo_by_id(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
        id: String,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let github_repo_account = next_account_info(account_info_iter)?;

        let repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;

        let (github_repo_pda_address, _bump) =
            Pubkey::find_program_address(&[b"repo_pda", id.as_bytes()], program_id);

        if github_repo_pda_address != *github_repo_account.key {
            msg!(
                "Provided public key ({:?}) does not match the derived PDA ({:?}).",
                github_repo_account.key,
                github_repo_pda_address
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

    /// Transfer reward from repo wallet to user wallet
    pub fn transfer_reward(accounts: &[AccountInfo], program_id: &Pubkey) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let payer = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;
        let user_account = next_account_info(account_info_iter)?;
        let user_wallet_account = next_account_info(account_info_iter)?;
        let pr_counter_account = next_account_info(account_info_iter)?;
        let repo_wallet_account = next_account_info(account_info_iter)?;

        if !payer.is_signer {
            msg!("payer is not a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut user_data = match User::try_from_slice(&user_account.data.borrow()) {
            Ok(data) => data,
            Err(_) => {
                msg!("Failed to deserialize User data.");
                return Err(ProgramError::InvalidArgument);
            }
        };

        let mut githup_repo_data =
            match GithubRepo::try_from_slice(&github_repo_account.data.borrow()) {
                Ok(data) => data,
                Err(_) => {
                    msg!("Failed to deserialize GithubRepo data.");
                    return Err(ProgramError::InvalidArgument);
                }
            };

        let mut prcount_data = match PrCount::try_from_slice(&pr_counter_account.data.borrow()) {
            Ok(data) => data,
            Err(_) => {
                msg!("Failed to deserialize PrCount data.");
                return Err(ProgramError::InvalidArgument);
            }
        };

        let (repo_wallet_pda, _wallet_bump) = Pubkey::find_program_address(
            &[b"repo_wallet", githup_repo_data.id.as_bytes()],
            program_id,
        );

        if repo_wallet_pda != *repo_wallet_account.key {
            msg!("Provided repo wallet account does not match derived PDA.");
            return Err(ProgramError::InvalidArgument);
        }
        // PR sayısı limiti aşıldı mı kontrol et
        if prcount_data.prcount >= githup_repo_data.pull_request_limit {
            **repo_wallet_account.try_borrow_mut_lamports()? -=
                githup_repo_data.reward_per_pull_request;
            **user_wallet_account.try_borrow_mut_lamports()? +=
                githup_repo_data.reward_per_pull_request;

            user_data.totalearn = user_data
                .totalearn
                .checked_add(githup_repo_data.reward_per_pull_request)
                .ok_or(ArithmeticErr)?;

            user_data.total_pr_count = user_data
                .total_pr_count
                .checked_add(githup_repo_data.pull_request_limit)
                .ok_or(ArithmeticErr)?;

            githup_repo_data.total_pull_requests = githup_repo_data
                .total_pull_requests
                .checked_add(githup_repo_data.pull_request_limit)
                .ok_or(ArithmeticErr)?;

            prcount_data.prcount = 0;
            msg!(
                "Transfer successful. Reward amount: {}",
                githup_repo_data.reward_per_pull_request
            );
        } else {
            msg!("Pull request limit has not been reached yet.");
        }

        prcount_data.serialize(&mut &mut pr_counter_account.try_borrow_mut_data()?[..])?;
        user_data.serialize(&mut &mut user_account.try_borrow_mut_data()?[..])?;
        githup_repo_data.serialize(&mut &mut github_repo_account.try_borrow_mut_data()?[..])?;

        Ok(())
    }

    /// Load bounty into the repo wallet
    pub fn load_bounty_repo(
        accounts: &[AccountInfo],
        _program_id: &Pubkey,
        data: LoudBountyAccount, // Yüklenecek SOL miktarı
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let phantom_wallet_account = next_account_info(account_info_iter)?;
        let github_repo_account = next_account_info(account_info_iter)?;

        if !phantom_wallet_account.is_signer {
            msg!("Phantom wallet account is not a signer");
            return Err(ProgramError::MissingRequiredSignature);
        }

        if !github_repo_account.is_writable {
            msg!("GitHub repo account is not writable");
            return Err(ProgramError::InvalidAccountData);
        }

        // Phantom wallet'tan Repo Wallet PDA adresine SOL transferi
        invoke(
            &system_instruction::transfer(
                phantom_wallet_account.key,
                github_repo_account.key,
                data.amount,
            ),
            &[phantom_wallet_account.clone(), github_repo_account.clone()],
        )?;

        msg!(
            "Loaded {} lamports into the repo wallet address: {:?}",
            data.amount,
            github_repo_account.key
        );

        Ok(())
    }

    /// Get pull requests per user
    pub fn get_pull_requests_per_user(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user = next_account_info(account_info_iter)?;

        // Kullanıcı verilerini oku
        let user_data = User::try_from_slice(&user.data.borrow())?;

        let account_info_vec: Vec<&AccountInfo> = account_info_iter.collect();

        // Kullanıcıya ait repoları döngü ile gez
        for chunk in account_info_vec.chunks(2) {
            let github_repo_account = chunk[0]; // İlk hesap GithubRepo hesabı
            let pr_count_account = chunk[1]; // İkinci hesap PR Count hesabı

            let repo_data = GithubRepo::try_from_slice(&github_repo_account.data.borrow())?;
            let pr_count_data = PrCount::try_from_slice(&pr_count_account.data.borrow())?;

            // Repo PDA'sını hesapla ve doğrula
            let (github_repo_pda_address, __bump) =
                Pubkey::find_program_address(&[b"repo_pda", repo_data.id.as_bytes()], program_id);

            if github_repo_pda_address != *github_repo_account.key {
                msg!("Invalid Repo PDA for repo: {}", repo_data.repo_url);
                continue; // Eğer PDA uyumsuzsa bu repo'yu atlayıp devam et
            }

            // PR Counter PDA'sını hesapla ve doğrula
            let (pr_counter_address, _bump) = Pubkey::find_program_address(
                &[
                    b"pull request counter",
                    user_data.github_username.as_ref(),
                    repo_data.repo_url.as_ref(),
                ],
                program_id,
            );

            if pr_counter_address != *pr_count_account.key {
                msg!(
                    "Invalid PR Count PDA for user: {} and repo: {}",
                    user_data.github_username,
                    repo_data.repo_url
                );
                continue; // Eğer PDA uyumsuzsa bu PR'yi atlayıp devam et
            }

            // Kullanıcı ve repo için PR sayısını ekrana yazdır
            msg!(
                "User: {}, Repo: {}, Pull Requests: {}",
                user_data.github_username,
                repo_data.repo_url,
                pr_count_data.prcount
            );
        }

        Ok(())
    }
}
