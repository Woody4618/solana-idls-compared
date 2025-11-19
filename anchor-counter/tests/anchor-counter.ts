import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorCounter } from "../target/types/anchor_counter";
import { expect } from "chai";

describe("anchor-counter", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.AnchorCounter as Program<AnchorCounter>;

  // Generate a new keypair for the counter account
  const counterKeypair = anchor.web3.Keypair.generate();

  const initialValue = 100;

  it("Initializes the counter", async () => {
    // Initialize the counter with an initial value
    const tx = await program.methods
      .initializeCounter(new anchor.BN(initialValue))
      .accounts({
        counter: counterKeypair.publicKey,
        authority: provider.wallet.publicKey,
      })
      .signers([counterKeypair])
      .rpc();

    console.log("Initialize transaction signature:", tx);

    // Fetch the counter account
    const counterAccount = await program.account.counter.fetch(
      counterKeypair.publicKey
    );

    // Verify the counter was initialized correctly
    expect(counterAccount.count.toNumber()).to.equal(initialValue);
    expect(counterAccount.authority.toString()).to.equal(
      provider.wallet.publicKey.toString()
    );

    console.log(
      "Counter initialized with value:",
      counterAccount.count.toNumber()
    );
  });

  it("Increments the counter", async () => {
    // Increment the counter
    const tx = await program.methods
      .incrementCounter()
      .accounts({
        counter: counterKeypair.publicKey,
        authority: provider.wallet.publicKey,
      })
      .rpc();

    console.log("Increment transaction signature:", tx);

    // Fetch the updated counter account
    const counterAccount = await program.account.counter.fetch(
      counterKeypair.publicKey
    );

    // Verify the counter was incremented
    expect(counterAccount.count.toNumber()).to.equal(initialValue + 1);

    console.log("Counter incremented to:", counterAccount.count.toNumber());
  });

  it("Increments the counter multiple times", async () => {
    // Increment 5 more times
    for (let i = 0; i < 5; i++) {
      await program.methods
        .incrementCounter()
        .accounts({
          counter: counterKeypair.publicKey,
          authority: provider.wallet.publicKey,
        })
        .rpc();
    }

    // Fetch the final counter value
    const counterAccount = await program.account.counter.fetch(
      counterKeypair.publicKey
    );

    // Should be initial (100) + 1 (from previous test) + 5 = 106
    expect(counterAccount.count.toNumber()).to.equal(initialValue + 6);

    console.log("Final counter value:", counterAccount.count.toNumber());
  });

  it("Performs CPI to native counter program", async () => {
    // Load the native program ID from its keypair file
    const fs = require("fs");
    const nativeProgramKeypair = JSON.parse(
      fs.readFileSync("../target/deploy/counter_program-keypair.json", "utf8")
    );
    const nativeProgramId = anchor.web3.Keypair.fromSecretKey(
      Uint8Array.from(nativeProgramKeypair)
    ).publicKey;

    console.log("Native Program ID:", nativeProgramId.toString());

    // Create a keypair for the native counter
    const nativeCounterKeypair = anchor.web3.Keypair.generate();

    // First, initialize a native counter using the native program
    // Native InitializeCounter instruction data:
    // - 1 byte discriminator (0 = InitializeCounter)
    // - 8 bytes initial_value (u64)
    const initValue = new anchor.BN(200);
    const initDiscriminator = Buffer.from([0]); // Variant 0
    const initValueBytes = initValue.toArrayLike(Buffer, "le", 8);
    const initInstructionData = Buffer.concat([
      initDiscriminator,
      initValueBytes,
    ]);

    const initNativeInstruction = new anchor.web3.TransactionInstruction({
      keys: [
        {
          pubkey: nativeCounterKeypair.publicKey,
          isSigner: true,
          isWritable: true,
        },
        { pubkey: provider.wallet.publicKey, isSigner: true, isWritable: true },
        {
          pubkey: anchor.web3.SystemProgram.programId,
          isSigner: false,
          isWritable: false,
        },
      ],
      programId: nativeProgramId,
      data: initInstructionData,
    });

    const initTx = new anchor.web3.Transaction().add(initNativeInstruction);
    await provider.sendAndConfirm(initTx, [nativeCounterKeypair]);

    console.log("Native counter initialized with value:", initValue.toNumber());

    // Read the initial native counter value
    const nativeCounterAccountBefore = await provider.connection.getAccountInfo(
      nativeCounterKeypair.publicKey
    );
    const countBefore = nativeCounterAccountBefore!.data.readBigUInt64LE(0);
    console.log("Native counter value before CPI:", countBefore.toString());

    // Now perform CPI from Anchor to increment the native counter
    const tx = await program.methods
      .incrementNativeCounter()
      .accounts({
        nativeCounter: nativeCounterKeypair.publicKey,
        nativeProgram: nativeProgramId,
      })
      .rpc();

    console.log("CPI to native program transaction signature:", tx);

    // Read the updated native counter value
    const nativeCounterAccountAfter = await provider.connection.getAccountInfo(
      nativeCounterKeypair.publicKey
    );
    const countAfter = nativeCounterAccountAfter!.data.readBigUInt64LE(0);

    // Verify the counter was incremented
    expect(Number(countAfter)).to.equal(initValue.toNumber() + 1);

    console.log("✅ CPI Test Passed!");
    console.log("Native counter value after CPI:", countAfter.toString());
  });
});
