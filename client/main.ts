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
  import {deserialize, deserializeUnchecked, serialize } from "borsh";
  import {User,UserShema,GithubRepo,GithubRepoShema,PrCount,PrCountShema} from "./models";
  const connection = new Connection("https://api.devnet.solana.com", "confirmed");

  const privatekey = [255,198,26,124,50,247,86,155,237,155,233,203,4,75,223,162,218,242,132,212,91,59,71,20,139,120,96,231,206,190,27,226,85,199,71,164,51,152,9,42,4,163,229,116,27,107,216,117,245,194,60,57,158,221,79,221,47,130,60,9,175,141,162,150]
  const payer = Keypair.fromSecretKey(Uint8Array.from(privatekey));
 
  const program_id =  new PublicKey("2jAKMfJuVK1smALRFnQSGs1Bc1qhCDbUBY8m7ZoLZ6Kd"); 
  const pr_count = new PublicKey("");

  const total_pull_request_count = async() => {
    const total_pr_count = new User();
    total_pr_count.total_pr_count = BigInt(0);

    const encoded = serialize(UserShema,total_pr_count);
    const concat = Uint8Array.of(0, ...encoded);

    const counterPDA = PublicKey.findProgramAddressSync([Buffer.from("total pull request counter")],program_id)

    const instruction = new TransactionInstruction({
      keys: [
        {pubkey: payer.publicKey, isSigner: true, isWritable: true},
        {pubkey: counterPDA[0], isSigner: false, isWritable: true}, // counterPDA[0]-> publickey counterPDA[1] -> bump dondurur
        {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
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


  const pull_request_count = async() => {
    const pr_count = new PrCount();
    pr_count.prcount = BigInt(0);

    const encoded = serialize(UserShema,pr_count);
    const concat = Uint8Array.of(3, ...encoded);

    const prCountPDA = PublicKey.findProgramAddressSync([Buffer.from("pull request counter")],program_id)

    const instruction = new TransactionInstruction({
      keys: [
        {pubkey: payer.publicKey, isSigner: true, isWritable: true},
        {pubkey: prCountPDA[0], isSigner: false, isWritable: true}, 
        {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},
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

  const create_user = async(github_username: string, phantom_wallet: Uint8Array) => {
    const user = new User();
    user.github_username = github_username.toString();
    user.phantom_wallet = phantom_wallet;
    user.totalearn = BigInt(0);
    user.submitted_at = BigInt(0);
    user.total_pr_count = BigInt(0);

    const encoded = serialize(UserShema, user);
    const concat = Uint8Array.of(1, ...encoded);

    const userPDA = PublicKey.findProgramAddressSync([Buffer.from("user_pda"), Buffer.from(github_username)], program_id);


    const instruction = new TransactionInstruction({
      keys: [
        {pubkey: payer.publicKey, isSigner: true, isWritable: true},
        {pubkey: userPDA[0], isSigner: false, isWritable: true},
        {pubkey: SystemProgram.programId, isSigner: false, isWritable: false},

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

  //kullancii getit ekle -> kullaniciyi donsun 