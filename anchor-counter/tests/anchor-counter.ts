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
        authority: provider.wallet.publicKey
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
    
    console.log("Counter initialized with value:", counterAccount.count.toNumber());
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
});
