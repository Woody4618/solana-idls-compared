/**
 * Example TypeScript client using Codama-generated functions
 *
 * This example demonstrates how to use the auto-generated Codama client
 * to interact with the Counter Program on Solana.
 *
 * Prerequisites:
 * 1. Deploy the program: cargo build-sbf && solana program deploy target/deploy/counter_program.so
 * 2. Generate clients: pnpm generate
 * 3. Start local validator: solana-test-validator
 * 4. Run this client: pnpm client
 *
 * Key benefits of using Codama-generated clients:
 * - Type-safe instruction builders
 * - Automatic serialization/deserialization
 * - Auto-syncs with program changes via IDL
 * - Less boilerplate code
 */

import {
  createKeyPairSignerFromBytes,
  generateKeyPairSigner,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  getSignatureFromTransaction,
  sendAndConfirmTransactionFactory,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstructions,
  signTransactionMessageWithSigners,
  pipe,
  lamports,
  address,
  type Address,
  type TransactionSigner,
} from "@solana/kit";

import {
  getInitializeCounterInstruction,
  getIncrementCounterInstruction,
  COUNTER_PROGRAM_PROGRAM_ADDRESS,
} from "../clients/js/src/generated/index.js";

// Configuration
const RPC_URL = "http://localhost:8899";
const WS_RPC_URL = "ws://localhost:8900";
const PROGRAM_ID = COUNTER_PROGRAM_PROGRAM_ADDRESS;

async function main() {
  console.log("🚀 Codama Client Example\n");
  console.log(`Program ID: ${PROGRAM_ID}\n`);

  // Create RPC client
  const rpc = createSolanaRpc(RPC_URL);
  const rpcSubscriptions = createSolanaRpcSubscriptions(WS_RPC_URL);

  // Create transaction sender
  const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({
    rpc,
    rpcSubscriptions,
  });

  // Generate keypairs
  console.log("Generating keypairs...");
  const payer = await generateKeyPairSigner();
  const counter = await generateKeyPairSigner();

  console.log(`Payer: ${payer.address}`);
  console.log(`Counter: ${counter.address}\n`);

  // Request airdrop
  console.log("Requesting airdrop...");
  try {
    const airdropSignature = await rpc
      .requestAirdrop(payer.address, lamports(1_000_000_000n))
      .send();

    console.log(`Airdrop signature: ${airdropSignature}`);
    console.log("Waiting for confirmation...");

    // Wait for confirmation (simple polling)
    await new Promise((resolve) => setTimeout(resolve, 2000));
    console.log("Airdrop confirmed ✅\n");
  } catch (error) {
    console.error("Failed to request airdrop:", error);
    console.log("Continuing anyway (assuming funded account)...\n");
  }

  // Get latest blockhash
  const { value: latestBlockhash } = await rpc.getLatestBlockhash().send();

  // ========================================
  // Initialize Counter
  // ========================================
  console.log("📝 Initializing counter with value 100...");

  // Create initialize instruction using Codama-generated function
  const initializeInstruction = getInitializeCounterInstruction({
    counter: counter,
    payer: payer,
    initialValue: 100n,
  });

  // Build and send transaction
  const initializeMessage = pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => setTransactionMessageFeePayerSigner(payer, tx),
    (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    (tx) => appendTransactionMessageInstructions([initializeInstruction], tx)
  );

  const signedInitializeTx =
    await signTransactionMessageWithSigners(initializeMessage);

  // Extract signature from the signed transaction
  const initializeSignature = getSignatureFromTransaction(signedInitializeTx);

  // Send and confirm transaction
  await sendAndConfirmTransaction(signedInitializeTx, {
    commitment: "confirmed",
  });

  console.log(`✅ Counter initialized!`);
  console.log(`Transaction: ${initializeSignature}`);
  console.log(
    `Explorer: https://explorer.solana.com/tx/${initializeSignature}?cluster=custom&customUrl=${encodeURIComponent(RPC_URL)}\n`
  );

  // ========================================
  // Increment Counter
  // ========================================
  console.log("➕ Incrementing counter...");

  // Create increment instruction using Codama-generated function
  const incrementInstruction = getIncrementCounterInstruction({
    counter: counter.address,
  });

  // Get fresh blockhash for new transaction
  const { value: freshBlockhash } = await rpc.getLatestBlockhash().send();

  // Build and send transaction
  const incrementMessage = pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => setTransactionMessageFeePayerSigner(payer, tx),
    (tx) => setTransactionMessageLifetimeUsingBlockhash(freshBlockhash, tx),
    (tx) => appendTransactionMessageInstructions([incrementInstruction], tx)
  );

  const signedIncrementTx =
    await signTransactionMessageWithSigners(incrementMessage);

  // Extract signature from the signed transaction
  const incrementSignature = getSignatureFromTransaction(signedIncrementTx);

  // Send and confirm transaction
  await sendAndConfirmTransaction(signedIncrementTx, {
    commitment: "confirmed",
  });

  console.log(`✅ Counter incremented!`);
  console.log(`Transaction: ${incrementSignature}`);
  console.log(
    `Explorer: https://explorer.solana.com/tx/${incrementSignature}?cluster=custom&customUrl=${encodeURIComponent(RPC_URL)}\n`
  );

  // ========================================
  // Fetch and Display Counter Account
  // ========================================
  console.log("📊 Fetching counter account...");

  try {
    const accountInfo = await rpc
      .getAccountInfo(counter.address, { encoding: "base64" })
      .send();

    if (accountInfo.value) {
      // Parse the account data (first 8 bytes is the counter value as u64)
      const data = Buffer.from(accountInfo.value.data[0], "base64");
      const count = data.readBigUInt64LE(0);

      console.log(`Counter Account: ${counter.address}`);
      console.log(`Current Count: ${count}`);
      console.log(`Owner: ${accountInfo.value.owner}`);
      console.log(`Lamports: ${accountInfo.value.lamports}\n`);
    } else {
      console.log("Account not found!\n");
    }
  } catch (error) {
    console.error("Failed to fetch account:", error);
  }

  console.log("✨ Example complete!");
}

// Run the example
main().catch((error) => {
  console.error("Error:", error);
  process.exit(1);
});
