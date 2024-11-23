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
  SYSVAR_RENT_PUBKEY,

} from "@solana/web3.js";
import { deserialize, serialize } from "borsh";
import { User, UserShema, GithubRepo, GithubRepoShema, PrCount, PrCountShema, UserForCreate, UserForCreateShema, PrCountAccess, PrCountAccessShema } from "./models";
const connection = new Connection("https://api.devnet.solana.com/", "confirmed");

const privatekey = [96, 105, 112, 230, 111, 23, 182, 37, 224, 241, 51, 108, 76, 156, 240, 180, 3, 209, 232, 107, 148, 38, 252, 171, 79, 6, 53, 220, 154, 195, 76, 79, 29, 243, 93, 105, 64, 148, 53, 217, 112, 192, 90, 18, 120, 45, 250, 253, 196, 5, 196, 123, 226, 88, 239, 5, 225, 17, 12, 23, 143, 232, 58, 107]
const payer = Keypair.fromSecretKey(Uint8Array.from(privatekey));

const program_id = new PublicKey("FEu3sURKJ32B1KpcdqkesAfznP8P4tZW3WUh3icaSKsf");

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

  const encoded = serialize(GithubRepoShema, repo);
  const concat = Uint8Array.of(4, ...encoded);

  const githubrepoPDA = PublicKey.findProgramAddressSync([Buffer.from("repo_pda"), Buffer.from(repo.id)], program_id);
  const repoWalletPDA = PublicKey.findProgramAddressSync([Buffer.from("repo_wallet"), Buffer.from(repo.id)], program_id);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: githubrepoPDA[0], isSigner: false, isWritable: true },
      { pubkey: repoWalletPDA[0], isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }, // System Program

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

  await connection.sendTransaction(tx);

  console.log("New Repository account => " + githubrepoPDA[0])
  console.log("New Repo Wallet account => " + repoWalletPDA[0]);
}

const getRepo = async (id: string): Promise<GithubRepo> => {
  const publicKey = PublicKey.findProgramAddressSync([Buffer.from("repo_pda"), Buffer.from(id)], program_id);
  const repo_read = await connection.getAccountInfo(publicKey[0]);

  if (repo_read == null) {
    return new GithubRepo();
  }
  const repo_deserialized = deserialize(GithubRepoShema, GithubRepo, repo_read.data);
  const wallet = new PublicKey(repo_deserialized.repo_wallet_address);
  console.log(wallet.toString());
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

    }
  }

  console.log("All repos:", githubRepos);
  return githubRepos;
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

    return true;
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

    return true;
  }

}

const getCuttentPRCount = async (
  user: PublicKey,
  githubRepoId: string) => {
  const prCounterPDA = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pull request counter"),
      Buffer.from(user.toBytes()),
      Buffer.from(githubRepoId)
    ],
    program_id
  );

  const prCount = await connection.getAccountInfo(prCounterPDA[0]);

  if (prCount == null) {
    return BigInt(0);
  }

  const prCountDeserialize = deserialize(PrCountShema, PrCount, prCount.data);

  console.log("Şimdiki Sayaç->", prCountDeserialize.prcount.toLocaleString());
}

const transferReward = async (
  id: string,
  phantomWallet: PublicKey
) => {

  const githubRepoPDA = PublicKey.findProgramAddressSync([Buffer.from("repo_pda"), Buffer.from(id)], program_id);

  const userPDA = PublicKey.findProgramAddressSync(
    [Buffer.from("user_pda"), Buffer.from(phantomWallet.toBytes())],
    program_id
  );

  const prCounterPDA = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pull request counter"),
      Buffer.from(phantomWallet.toBytes()),
      Buffer.from(id)
    ],
    program_id
  );

  const repoWalletPda = PublicKey.findProgramAddressSync([Buffer.from("repo_wallet"), Buffer.from(id)], program_id);

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: payer.publicKey, isSigner: true, isWritable: true },
      { pubkey: githubRepoPDA[0], isSigner: false, isWritable: true },
      { pubkey: userPDA[0], isSigner: false, isWritable: true },
      { pubkey: phantomWallet, isSigner: false, isWritable: true },
      { pubkey: prCounterPDA[0], isSigner: false, isWritable: true },
      { pubkey: repoWalletPda[0], isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
    ],
    data: Buffer.from([7]),
    programId: program_id
  });

  const latestBlockhash = await connection.getLatestBlockhash();
  const message = new TransactionMessage({
    instructions: [instruction],
    payerKey: payer.publicKey,
    recentBlockhash: latestBlockhash.blockhash
  }).compileToV0Message();


  const transaction = new VersionedTransaction(message);
  transaction.sign([payer]);

  const txSignature = await connection.sendTransaction(transaction);

  console.log("Transfer işlemi başarılı. TX Signature:", txSignature);


}

const loadBountyRepo = async (
  id: string,
  phantomWallet: PublicKey,
  amount: bigint,
) => {
  const data = Buffer.alloc(8); // 64-bit unsigned integer için 8 byte ayırıyoruz
  data.writeBigUInt64LE(amount); // Miktarı little-endian olarak yazıyoruz

  const github_repo_account = PublicKey.findProgramAddressSync([Buffer.from("repo_pda"), Buffer.from("12345")], program_id);
  const owner_wallet_account = new PublicKey("BUBtN9W8Ypt7S1w5otZVM7cU8HTgM7M2CjTt4z1L1Net");

  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: phantomWallet, isSigner: true, isWritable: true },
      { pubkey: github_repo_account[0], isSigner: false, isWritable: true },
      { pubkey: owner_wallet_account, isSigner: false, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
    ],
    data: data,
    programId: program_id
  });

  const latestBlockhash = await connection.getLatestBlockhash();
  const message = new TransactionMessage({
    instructions: [instruction],
    payerKey: phantomWallet,
    recentBlockhash: latestBlockhash.blockhash
  }).compileToV0Message();


  const transaction = new VersionedTransaction(message);

  const txSignature = await connection.sendTransaction(transaction);

  console.log("Bounty yükleme işlemi başarılı. TX Signature:", txSignature);
}

const getRepoBalace = async (
  id: string,              // Repo ID
) => {

  // 1. GitHub repo PDA'sını oluştur
  const githubRepoPDA = PublicKey.findProgramAddressSync([Buffer.from("repo_wallet"), Buffer.from(id)], program_id);


  const wallet = new PublicKey(githubRepoPDA[0]);
  const balanceLamports = await connection.getBalance(wallet);

  // 2. Bakiyeyi SOL cinsine çevir (1 SOL = 1,000,000,000 lamports)
  return balanceLamports / LAMPORTS_PER_SOL;
}

(async () => {
  // var repos: GithubRepo[] = await getAllRepos();

  // for (let repo of repos) {
  //   console.log(repo);
  // }


})();



(async () => {
  console.log("Starting...");
  const wallet1 = new PublicKey("C6nfQf35zJZ4mw1kCGYSSm9NjhyQi9K74fLGnhZqTpPJ")
  // create_user("edanur-caglayann", wallet1.toBytes());


  const wallet2 = new PublicKey("BUBtN9W8Ypt7S1w5otZVM7cU8HTgM7M2CjTt4z1L1Net")


  // const repo = new GithubRepo();
  // repo.id = "12345";
  // repo.repo_name = "deneme with account";
  // repo.repo_description = "deneme";
  // repo.repo_url = "https://github.com/deneme";
  // repo.total_pull_requests = BigInt(0);
  // repo.pull_request_limit = BigInt(3);
  // repo.reward_per_pull_request = BigInt(1);
  // repo.owner_wallet_address = wallet2.toBytes();
  // repo.repo_wallet_address = new Uint8Array(32);

  // await create_repo(repo);
  var result = await getRepo("12345");
  console.log(result);

  await loadBountyRepo("12345", wallet1, BigInt(0.5 * 1_000_000_000)); // 0.5 SOL
  console.log(await getRepo("12345"));

  // const result = await getRepoBalace("12345");
  // console.log("Repo balance:", result);


  // console.log("Creating repo...");
  // await create_repo(repo);

  //console.log(await getRepo("12345"));
  //await getUser(wallet1.toBytes());
  // console.log("Increasing PR count...");

  //await increasePullRequestCount(wallet1, "12345");
  // console.log("Increasing PR count...");

  // await increasePullRequestCount(wallet1, "123");

  // console.log("Increasing PR count...");
  //await increasePullRequestCount(wallet1, "12345");
  //
  //await sleep(6000);

  // console.log("Getting PR count...");
  //await getCuttentPRCount(wallet1, "12345");

  // console.log("Transfering reward...");
  //  await transferReward("12345", wallet1);

  // console.log("Getting user info...");
  // console.log(await getUser(wallet1.toBytes()));

}
)();

async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
