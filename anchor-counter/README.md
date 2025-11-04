# Anchor Counter Program

A Solana counter program built with the Anchor framework. This is the Anchor version of the native counter program in the parent directory.

## 📋 Features

- **Initialize Counter** - Create a new counter with an initial value
- **Increment Counter** - Increment the counter by 1 with overflow protection

## 🏗️ Program Structure

```
programs/anchor-counter/src/
└── lib.rs              # Main program with instructions and account definitions
```

### Instructions

#### `initialize_counter`

Creates a new counter account with an initial value.

**Accounts:**

- `counter` - The counter account to initialize (writable, signer)
- `authority` - Account that will own the counter (writable, signer, pays for creation)
- `system_program` - System program (auto-included)

**Arguments:**

- `initial_value: u64` - Starting value for the counter

#### `increment_counter`

Increments the counter by 1.

**Accounts:**

- `counter` - The counter account to increment (writable)
- `authority` - Authority that can modify the counter (signer)

**Arguments:** None

### Account Structure

**Counter**

```rust
pub struct Counter {
    pub count: u64,        // Current counter value
    pub authority: Pubkey, // Owner of this counter
}
```

### Errors

- `CounterOverflow` (6000) - Occurs when incrementing would cause overflow

## 🚀 Quick Start

### Install Dependencies

```bash
npm install
```

### Build

```bash
anchor build
```

### Test

```bash
anchor test
```

### Deploy

You can change the cluster in the Anchor.toml file or pass it as parameter.

```bash
# Deploy to localnet (start local validator first)
anchor deploy

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet-beta
anchor deploy --provider.cluster mainnet-beta
```

## 📝 Development

### Build Program

```bash
anchor build
```

This will:

- Compile the program
- Generate the IDL at `target/idl/anchor_counter.json`
- Generate TypeScript types at `target/types/anchor_counter.ts`

### Run Tests

```bash
anchor test
```

Tests are located in `tests/anchor-counter.ts` and include:

- Initializing a counter with value 100
- Incrementing the counter once
- Incrementing the counter multiple times

### Get Program ID

```bash
anchor keys list
```

Or from the deployed keypair:

```bash
solana address -k target/deploy/anchor_counter-keypair.json
```

## 🔑 Program ID

The program ID is declared in:

- `programs/anchor-counter/src/lib.rs`
- `Anchor.toml`

Current program ID: `27YreJqker2o5TvzzLUsiC9ZGMdPThvEm8qZBNDw5EWX`

## 📊 Comparison with Native Program

| Feature                | Anchor                          | Native                     |
| ---------------------- | ------------------------------- | -------------------------- |
| **Boilerplate**        | Minimal (Anchor handles it)     | Manual (more verbose)      |
| **Account Validation** | Automatic via macros            | Manual validation          |
| **IDL Generation**     | Automatic                       | Manual with Codama         |
| **Type Safety**        | TypeScript types auto-generated | Requires client generation |
| **Security**           | Built-in checks                 | Manual implementation      |
| **Size**               | Slightly larger                 | Smaller                    |
| **Development Speed**  | Faster                          | Slightly more control      |

## 🔧 Configuration

### Anchor.toml

```toml
[programs.localnet]
anchor_counter = "27YreJqker2o5TvzzLUsiC9ZGMdPThvEm8qZBNDw5EWX"

[provider]
cluster = "Localnet"
wallet = "~/.config/solana/id.json"

[scripts]
test = "yarn run ts-mocha -p ./tsconfig.json -t 1000000 tests/**/*.ts"
```

## 📦 Generated Artifacts

After building, you'll find:

```
target/
├── deploy/
│   ├── anchor_counter.so           # Compiled program
│   └── anchor_counter-keypair.json # Program keypair
├── idl/
│   └── anchor_counter.json         # Interface Definition Language
└── types/
    └── anchor_counter.ts           # TypeScript types
```

## 🧪 Using the Program

### TypeScript Client Example

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AnchorCounter } from "../target/types/anchor_counter";

const program = anchor.workspace.AnchorCounter as Program<AnchorCounter>;
const counterKeypair = anchor.web3.Keypair.generate();

// Initialize counter
await program.methods
  .initializeCounter(new anchor.BN(100))
  .accounts({
    counter: counterKeypair.publicKey,
    authority: wallet.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([counterKeypair])
  .rpc();

// Increment counter
await program.methods
  .incrementCounter()
  .accounts({
    counter: counterKeypair.publicKey,
    authority: wallet.publicKey,
  })
  .rpc();

// Fetch counter
const counter = await program.account.counter.fetch(counterKeypair.publicKey);
console.log("Count:", counter.count.toNumber());
```

## 📖 Learn More

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Anchor Book](https://book.anchor-lang.com/)
- [Solana Cookbook](https://solanacookbook.com/)

## 📄 License

MIT
