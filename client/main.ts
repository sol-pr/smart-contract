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

const privatekey =[96,105,112,230,111,23,182,37,224,241,51,108,76,156,240,180,3,209,232,107,148,38,252,171,79,6,53,220,154,195,76,79,29,243,93,105,64,148,53,217,112,192,90,18,120,45,250,253,196,5,196,123,226,88,239,5,225,17,12,23,143,232,58,107]
const payer = Keypair.fromSecretKey(Uint8Array.from(privatekey));

const program_id = new PublicKey("9ZAjGgwqtHQtjM6E1V31bHGZQqkvBHVbbEmmEgVU7fkC");
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
  const concat = Uint8Array.of(3, ...encoded);

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
  const concat = Uint8Array.of(1, ...encoded);

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
  return user_deserialized.github_username.toString();
}

(async () => {
  // try {
    const pubkey = new PublicKey("BUBtN9W8Ypt7S1w5otZVM7cU8HTgM7M2CjTt4z1L1Net");
    const userName = "bgraokmsuh";
  //   const createUser = await create_user(userName, pubkey.toBytes());

  //   console.log(createUser);
  // } catch (error) {
  //   console.log(error);

  // }

  const user = await getUser(pubkey.toBytes());
  console.log(user);
})();

const pubkey = new PublicKey("BUBtN9W8Ypt7S1w5otZVM7cU8HTgM7M2CjTt4z1L1Net");
// create_user("bgraokmsuh", pubkey.toBytes());

