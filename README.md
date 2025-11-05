# Counter Program - Solana with Codama

A Solana program demonstrating counter functionality with automatic IDL generation using Codama macros.

## 📋 Prerequisites

- [Rust](https://rustup.rs/) (with cargo)
- [Solana CLI Tools](https://docs.solanalabs.com/cli/install) (for `cargo build-sbf`)
- [Node.js](https://nodejs.org/) (v18+)
- [pnpm](https://pnpm.io/) (for client generation)

## 🏗️ Project Structure

```
.
├── src/
│   ├── lib.rs              # Program entry point
│   ├── processor.rs        # Instruction processing logic
│   ├── instructions.rs     # Instruction definitions (with Codama macros)
│   ├── state.rs            # Account state (with Codama macros)
│   └── errors.rs           # Error definitions (with Codama macros)
├── build.rs                # Auto-generates IDL at compile time
├── idl.json                # Generated IDL (created by build.rs)
├── codama.json             # Codama client generation config
├── package.json            # Node.js scripts for client generation
├── clients/
│   ├── js/                 # Generated TypeScript client
│   └── rust/               # Generated Rust client
└── examples/
    └── client.rs           # Example Rust client usage
```

## 🚀 Commands

### Building the Program

```bash
# Build for Solana deployment (SBF target)
cargo build-sbf
```

> **Note:** The `build.rs` script automatically generates `idl.json` before every build.

### Testing

```bash
# Run all tests
cargo test
```

### Generating Clients

First, install dependencies:

```bash
pnpm install
```

Then generate clients:

```bash
# Generate both TypeScript and Rust clients
pnpm generate

# Generate only TypeScript client
pnpm generate:js

# Generate only Rust client
pnpm generate:rust
```

Generated clients will be in:

- TypeScript: `clients/js/src/generated/`
- Rust: `clients/rust/`

### Deployment

```bash
# Deploy to localnet (start local validator first)
solana program deploy target/deploy/counter_program.so

# Deploy to devnet
solana program deploy target/deploy/counter_program.so --url devnet

# Get program ID
solana address -k target/deploy/counter_program-keypair.json
```

### Local Development

```bash
# Start local validator
solana-test-validator

# Check your SOL balance
solana balance

# Airdrop SOL for testing (localnet/devnet)
solana airdrop 2
```

## 📝 Instructions

The program supports these instructions:

### 1. Initialize Counter

Creates a new counter account with an initial value.

**Accounts:**

- `counter` - The counter account to create (writable, signer)
- `payer` - Account paying for creation (writable, signer)
- `system_program` - System program (default)

**Arguments:**

- `initial_value: u64` - Starting counter value

### 2. Increment Counter

Increments the counter by 1.

**Accounts:**

- `counter` - The counter account (writable)

**Arguments:** None

## 🔄 Development Workflow

### 1. Make code changes

Edit files in `src/`:

```bash
vim src/instructions.rs  # Add new instruction
```

### 2. Build (auto-generates IDL)

```bash
cargo build-sbf
```

This automatically:

- Runs `build.rs`
- Generates updated `idl.json`
- Compiles the program

### 3. Generate clients

```bash
pnpm generate
```

Creates TypeScript and Rust clients from the updated IDL.

### 4. Test

```bash
cargo test
```

### 5. Deploy

```bash
solana program deploy target/deploy/counter_program.so
```

### 6. Upload IDL

```bash
solana address -k target/deploy/counter_program-keypair.json

npx @solana-program/program-metadata@latest write idl <progam-id>  ./idl.json
```

## 💻 Example Clients

This project includes two example clients demonstrating different approaches:

### 1. Manual Rust Client (`examples/client.rs`)

A Rust client that manually constructs instructions using borsh serialization:

```bash
# Build and run (requires local validator)
cargo run --example client
```

**Features:**

- Uses `solana-client` and `solana-sdk`
- Manual instruction construction
- Direct borsh serialization
- Good for understanding low-level details

### 2. Codama TypeScript Client (`examples/codama-client.ts`)

A TypeScript client using the auto-generated Codama functions:

```bash
# Start local validator first
solana-test-validator

# Run the client
pnpm client
```

**Features:**

- Uses `@solana/kit` (web3.js 2.0)
- Auto-generated instruction builders
- Type-safe with full TypeScript support
- Cleaner, more maintainable code
- Leverages Codama's IDL-based code generation

**Example usage:**

```typescript
import {
  getInitializeCounterInstruction,
  getIncrementCounterInstruction,
} from "./clients/js/src/generated/index.js";

// Initialize counter with value 100
const initInstruction = getInitializeCounterInstruction({
  counter: counterKeypair,
  payer: payerKeypair,
  initialValue: 100n,
});

// Increment counter
const incrementInstruction = getIncrementCounterInstruction({
  counter: counterKeypair.address,
});
```

**Comparison:**

| Feature             | Manual Rust       | Codama TypeScript      |
| ------------------- | ----------------- | ---------------------- |
| **Type Safety**     | Compile-time      | Compile-time + Runtime |
| **Code Generation** | Manual            | Automatic              |
| **Maintainability** | Requires updates  | Auto-syncs with IDL    |
| **Learning Curve**  | Steeper           | Gentler                |
| **Use Case**        | Low-level control | Faster development     |

## 📚 Codama Macros

This project uses Codama derive macros for automatic IDL generation:

```rust
// State (accounts)
#[derive(CodamaAccount)]
pub struct CounterAccount { ... }

// Instructions
#[derive(CodamaInstructions)]
pub enum CounterInstruction { ... }

// Errors
#[derive(CodamaErrors)]
pub enum CounterError { ... }
```

## 🔧 Configuration

### Program ID

Set in `cargo.toml`:

```toml
[package.metadata.solana]
program-id = "ATjcKTRrFZwdTjSYpheKkEKKAPzf4iUoK6ZtPqJysnyN"
```

### Client Generation

Configure in `codama.json`:

```json
{
  "idl": "./idl.json",
  "scripts": {
    "js": [...],
    "rust": [...]
  }
}
```

### Build warnings about `unexpected_cfgs`?

These are harmless warnings from Solana's `entrypoint!` macro. They don't affect functionality and can be safely ignored.

## 🔍 Project Structure

```
solana-idls-compared/
├── src/                          # Native Solana program (Codama)
│   ├── lib.rs                   # Program entrypoint
│   ├── state.rs                 # Account definitions
│   ├── instructions.rs          # Instruction enum
│   ├── processor.rs             # Business logic
│   └── errors.rs                # Custom errors
├── anchor-counter/              # Anchor program (separate)
│   ├── programs/
│   │   └── anchor-counter/
│   │       └── src/lib.rs      # Anchor program
│   └── tests/                   # TypeScript tests
├── examples/
│   ├── client.rs               # Manual Rust client
│   └── codama-client.ts        # Generated TypeScript client
├── clients/                     # Auto-generated (from IDL)
│   ├── js/                     # TypeScript client
│   └── rust/                   # Rust client
├── build.rs                    # IDL generation script
├── idl.json                    # Generated IDL
├── codama.json                 # Client generation config
└── package.json                # Node.js scripts (pnpm)
```

## 🎯 Key Differences: Native vs Anchor vs Codama

| Aspect                       | Native Solana        | + Codama             | Anchor                  |
| ---------------------------- | -------------------- | -------------------- | ----------------------- |
| **Boilerplate**              | High                 | Medium               | Low                     |
| **IDL Generation**           | Manual               | Auto (build.rs)      | Auto (anchor build)     |
| **Client Generation**        | Manual               | Auto (Codama)        | Auto (Anchor)           |
| **Account Validation**       | Manual               | Manual               | Automatic               |
| **Type Safety**              | Rust only            | Rust + TS/JS         | Rust + TS/JS            |
| **Program Size**             | Small                | Small                | Slightly Larger         |
| **Learning Curve**           | Steep                | Medium               | Easy                    |
| **Control**                  | Maximum              | High                 | Medium                  |
| **Best For**                 | Special custom needs | Special custom needs | Production, prototyping |
| **Instruction Desciminator** | Custom               | Custom               | 8byte derived from name |

## 📖 Learn More

- [Native program quickstart](https://solana.com/de/docs/programs/rust/program-structure/)
- [Codama Documentation](https://github.com/codama-idl/codama)
- [Anchor Framework](https://www.anchor-lang.com/)
- [@solana/kit](https://www.solanakit.com/docs)

## 📄 License

MIT
