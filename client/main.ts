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
import { deserialize, serialize } from "borsh";
import { User, UserShema, GithubRepo, GithubRepoShema, PrCount, PrCountShema, UserForCreate, UserForCreateShema, CheckTransfer, CheckTransferShema, PrCountAccess, PrCountAccessShema } from "./models";
const connection = new Connection("https://api.devnet.solana.com", "confirmed");

const privatekey = [96, 105, 112, 230, 111, 23, 182, 37, 224, 241, 51, 108, 76, 156, 240, 180, 3, 209, 232, 107, 148, 38, 252, 171, 79, 6, 53, 220, 154, 195, 76, 79, 29, 243, 93, 105, 64, 148, 53, 217, 112, 192, 90, 18, 120, 45, 250, 253, 196, 5, 196, 123, 226, 88, 239, 5, 225, 17, 12, 23, 143, 232, 58, 107]
const payer = Keypair.fromSecretKey(Uint8Array.from(privatekey));

const program_id = new PublicKey("FEu3sURKJ32B1KpcdqkesAfznP8P4tZW3WUh3icaSKsf");
// const user = new PublicKey("Cqt5XDcKL3uw1ozwdFsretbGGHpDvsNLaYYhZgXXDCGZ");


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

  const repoPDA = PublicKey.findProgramAddressSync([Buffer.from("repo_pda"), Buffer.from(repo.id)], program_id);

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

const transferReward = async (
  githubRepoId: string,         // Repo ID
  githubUsername: string,       // Kullanıcının GitHub kullanıcı adı
  userPubkey: PublicKey
) => {

  // 1. GitHub repo PDA'sını oluştur
  const repoPDA = PublicKey.findProgramAddressSync(
    [Buffer.from("repo_pda"), Buffer.from(githubRepoId)],
    program_id
  );

  const repo_read = await connection.getAccountInfo(repoPDA[0]);
  if (repo_read == null) {
    console.error("Repo not found");
    return;
  }
  const repo_deserialized = deserialize(GithubRepoShema, GithubRepo, repo_read.data);
  const owner_wallet = new PublicKey(repo_deserialized.owner_wallet_address);

  // 2. Kullanıcının PR sayacı için PDA oluştur
  const prCounterPDA = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pull request counter"),
      Buffer.from(githubUsername),
      Buffer.from(githubRepoId)
    ],
    program_id
  );

  const data = Buffer.from([7]);


  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },          // Payer (işlemi başlatan)
      { pubkey: repoPDA[0], isSigner: false, isWritable: true },              // GitHub repo PDA
      { pubkey: prCounterPDA[0], isSigner: false, isWritable: true },         // PR sayaç PDA
      { pubkey: owner_wallet, isSigner: false, isWritable: true },       // Repo sahibinin cüzdan adresi
      { pubkey: userPubkey, isSigner: false, isWritable: true },              // PR yapan kullanıcının cüzdanı
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false } // Sistem programı (SOL transfer için)
    ],
    data: data,
    programId: program_id // Rust program ID'si
  });

  // 5. TransactionMessage oluştur
  const latestBlockhash = await connection.getLatestBlockhash(); // Blok hash alınması
  const message = new TransactionMessage({
    instructions: [instruction],
    payerKey: payer.publicKey,
    recentBlockhash: latestBlockhash.blockhash
  }).compileToV0Message();

  // 6. VersionedTransaction oluştur ve imzala
  const transaction = new VersionedTransaction(message);
  transaction.sign([payer]); // Payer işlemi imzalıyor

  // 7. Transaction'ı gönder
  const txSignature = await connection.sendTransaction(transaction);

  // 8. Transaction sonucu
  console.log("Transaction Signature:", txSignature);
  console.log("New PR Counter PDA:", prCounterPDA[0].toBase58());
  console.log("New Repository PDA:", repoPDA[0].toBase58());
}

const increasePullRequestCount = async (
  user: PublicKey,
  githubRepoId: string, // Repo ID'si
) => {
  //check account using pda
  const prCounterPDA = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pull request counter"),
      Buffer.from(user.toBytes()),
      Buffer.from(githubRepoId)
    ],
    program_id
  );

  const prCounterAccount = await connection.getAccountInfo(prCounterPDA[0]);
  if (prCounterAccount == null) {
    // create new account
    const prCountAccess = new PrCountAccess();
    prCountAccess.id = githubRepoId;
    prCountAccess.phantom_wallet = user.toBytes();

    //for create new account
    const encoded = serialize(PrCountAccessShema, prCountAccess);
    const concat = Uint8Array.of(0, ...encoded);

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: prCounterPDA[0], isSigner: false, isWritable: true },
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
    console.log("New users account => " + prCounterPDA[0])


  }
  else {
    const prCountAccess = new PrCountAccess();
    prCountAccess.id = githubRepoId;
    prCountAccess.phantom_wallet = user.toBytes();

    //for create new account
    const encoded = serialize(PrCountAccessShema, prCountAccess);
    const concat = Uint8Array.of(1, ...encoded);

    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        { pubkey: prCounterPDA[0], isSigner: false, isWritable: true },
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
    console.log("UpdatesUser => " + prCounterPDA[0])
  }

  const prCount = await connection.getAccountInfo(prCounterPDA[0]);

  if (prCount == null) {
    return BigInt(0);
  }

  const prCountDeserialize = deserialize(PrCountShema, PrCount, prCount.data);

  return prCountDeserialize.prcount;

};
(async () => {
  // const repos: GithubRepo[] = await getAllRepos();

  // repos.forEach(element => {
  //   console.log(element);
  // });

  const result = await increasePullRequestCount(new PublicKey("BUBtN9W8Ypt7S1w5otZVM7cU8HTgM7M2CjTt4z1L1Net"),
    "773002246");

  console.log("CurrentCount for bgraokmush =>", result);
}
)();

