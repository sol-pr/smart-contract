import {
  Connection,
  Keypair,
  PublicKey,
  TransactionMessage,
  VersionedTransaction,
  SystemProgram,
  TransactionInstruction,
  LAMPORTS_PER_SOL,
  Transaction,
  sendAndConfirmTransaction,

} from "@solana/web3.js";
import { deserialize, deserializeUnchecked, serialize } from "borsh";
import { User, UserShema, GithubRepo, GithubRepoShema, PrCount, PrCountShema, UserForCreate, UserForCreateShema } from "./models";
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

const privatekey = [96, 105, 112, 230, 111, 23, 182, 37, 224, 241, 51, 108, 76, 156, 240, 180, 3, 209, 232, 107, 148, 38, 252, 171, 79, 6, 53, 220, 154, 195, 76, 79, 29, 243, 93, 105, 64, 148, 53, 217, 112, 192, 90, 18, 120, 45, 250, 253, 196, 5, 196, 123, 226, 88, 239, 5, 225, 17, 12, 23, 143, 232, 58, 107]
const payer = Keypair.fromSecretKey(Uint8Array.from(privatekey));

const program_id = new PublicKey("FEu3sURKJ32B1KpcdqkesAfznP8P4tZW3WUh3icaSKsf");
// const user = new PublicKey("Cqt5XDcKL3uw1ozwdFsretbGGHpDvsNLaYYhZgXXDCGZ");


const total_pull_request_count = async () => {
  const total_pr_count = new User();
  total_pr_count.total_pr_count = BigInt(0);

  const encoded = serialize(UserShema, total_pr_count);
  const concat = Uint8Array.of(0, ...encoded);

  const counterPDA = PublicKey.findProgramAddressSync([Buffer.from("total pull request counter")], program_id)

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: counterPDA[0], isSigner: false, isWritable: true }, // counterPDA[0]-> publickey counterPDA[1] -> bump dondurur
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(concat),
    programId: program_id
  })

  const message = new TransactionMessage({
    instructions: [instruction],
    payerKey: payer.publicKey,
    recentBlockhash: (await connection.getLatestBlockhash()).blockhash
  }).compileToV0Message();


  const tx = new VersionedTransaction(message);
  tx.sign([payer]);

  connection.sendTransaction(tx);
  console.log("User Counter => " + counterPDA[0].toString())

}


const pull_request_count = async () => {
  const pr_count = new PrCount();
  pr_count.prcount = BigInt(0);

  const encoded = serialize(UserShema, pr_count);
  const concat = Uint8Array.of(1, ...encoded);

  const prCountPDA = PublicKey.findProgramAddressSync([Buffer.from("pull request counter")], program_id)

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: prCountPDA[0], isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: Buffer.from(concat),
    programId: program_id
  })

  const message = new TransactionMessage({
    instructions: [instruction],
    payerKey: payer.publicKey,
    recentBlockhash: (await connection.getLatestBlockhash()).blockhash
  }).compileToV0Message();


  const tx = new VersionedTransaction(message);
  tx.sign([payer]);

  connection.sendTransaction(tx);
  console.log("Pr Counter => " + prCountPDA[0].toString())

}

const create_user = async (github_username: string, phantom_wallet: Uint8Array) => {

  const userCreate = new UserForCreate();
  userCreate.github_username = github_username;
  userCreate.phantom_wallet = phantom_wallet;

  const encoded = serialize(UserForCreateShema, userCreate);
  const concat = Uint8Array.of(2, ...encoded);

  const userPDA = PublicKey.findProgramAddressSync([Buffer.from("user_pda"), Buffer.from(phantom_wallet)], program_id);


  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: userPDA[0], isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },

    ],
    data: Buffer.from(concat),
    programId: program_id
  })

  const message = new TransactionMessage({
    instructions: [instruction],
    payerKey: payer.publicKey,
    recentBlockhash: (await connection.getLatestBlockhash()).blockhash
  }).compileToV0Message();


  const tx = new VersionedTransaction(message);
  tx.sign([payer]);

  connection.sendTransaction(tx);
  console.log("New users account => " + userPDA[0])
}

const getUser = async (phantomWallet: Uint8Array): Promise<string> => {

  const publicKey = PublicKey.findProgramAddressSync([Buffer.from("user_pda"), Buffer.from(phantomWallet)], program_id);
  const user_read = await connection.getAccountInfo(publicKey[0]);

  if (user_read == null) {
    return "kullanici bulunamadi";
  }
  const user_deserialized = deserialize(UserShema, User, user_read.data);

  console.log(user_deserialized);
  return user_deserialized.github_username.toString();
}


const create_repo = async (repo: GithubRepo) => {
  const createRepo = new GithubRepo();
  createRepo.repo_url = repo.repo_url;
  createRepo.repo_name = repo.repo_name;
  createRepo.repo_description = repo.repo_description;
  createRepo.total_pull_requests = BigInt(0);
  createRepo.pull_request_limit = repo.pull_request_limit;
  createRepo.reward_per_pull_request = repo.reward_per_pull_request;
  createRepo.owner_wallet_address = repo.owner_wallet_address;

  const encoded = serialize(GithubRepoShema, createRepo);
  const concat = Uint8Array.of(4, ...encoded);

  const repoPDA = PublicKey.findProgramAddressSync([Buffer.from("repo_pda"), Buffer.from(repo.repo_url)], program_id);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: repoPDA[0], isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },

    ],
    data: Buffer.from(concat),
    programId: program_id
  });

  const message = new TransactionMessage({
    instructions: [instruction],
    payerKey: payer.publicKey,
    recentBlockhash: (await connection.getLatestBlockhash()).blockhash
  }).compileToV0Message();


  const tx = new VersionedTransaction(message);
  tx.sign([payer]);

  connection.sendTransaction(tx);
  console.log("New Repository account => " + repoPDA[0])

}

const getRepo = async (id: string): Promise<GithubRepo> => {
  const publicKey = PublicKey.findProgramAddressSync([Buffer.from("repo_pda"), Buffer.from(id)], program_id);
  const repo_read = await connection.getAccountInfo(publicKey[0]);

  if (repo_read == null) {
    return new GithubRepo();
  }
  const repo_deserialized = deserialize(GithubRepoShema, GithubRepo, repo_read.data);
  return repo_deserialized;
}


const getAllRepos = async () => {
  const accounts = await connection.getProgramAccounts(program_id);

  const githubRepos: GithubRepo[] = [];

  for (let account of accounts) {
    // Repo PDA adresini ve veriyi kontrol etmek için deserialize et
    try {
      const repoData = deserialize(
        GithubRepoShema,
        GithubRepo,
        account.account.data
      );

      githubRepos.push(repoData);
    } catch (err) {
      console.error('Error deserializing repo data:', err);
    }
  }

  console.log("All repos:", githubRepos);
  return githubRepos;
}


(async () => {
  //  try {
  // const pubkey = new PublicKey("C6nfQf35zJZ4mw1kCGYSSm9NjhyQi9K74fLGnhZqTpPJ");
  // const userName: string = "sol-pr";
  // //const createUser = await create_user(userName, pubkey.toBytes());

  // //   console.log(createUser);
  // // } catch (error) {
  // //   console.log(error);

  // //}

  // const user = await getUser(pubkey.toBytes());
  // console.log(user);

  //create repo

  // const repo = new GithubRepo();
  // repo.repo_url = "https:gitub.com/solana";
  // repo.repo_name = "solana";
  // repo.repo_description = "solana repo";
  // repo.pull_request_limit = BigInt(5);
  // repo.reward_per_pull_request = BigInt(1);
  // repo.total_pull_requests = BigInt(0);
  // repo.owner_wallet_address = pubkey.toBytes();

  // const response = await create_repo(repo);

  // console.log(response);

  // const repo = await getRepo("https:gitub.com/solana");

  // console.log(repo);


  const allRepos = await getAllRepos();

  console.log(allRepos);

})();

