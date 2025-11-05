# Codama Integration Guide

This document provides detailed information about using Codama for IDL generation and client code generation in native Solana programs.

Note: Codama macros are still in development and are not yet stable.

## 📋 Table of Contents

- [What is Codama?](#what-is-codama)
- [Setup](#setup)
- [Codama Macros](#codama-macros)
- [IDL Generation](#idl-generation)
- [Client Generation](#client-generation)
- [Instructions Documentation](#instructions-documentation)
- [Development Workflow](#development-workflow)
- [Troubleshooting](#troubleshooting)

## What is Codama?

Codama is a tool that describes Solana programs in a standardized format (Codama IDL) and enables:

- ✨ **Automatic client generation** in TypeScript and Rust
- 🔍 **Type-safe instruction builders**

## Setup

### 1. Add Dependencies

Add to `cargo.toml` (**see [cargo.toml](cargo.toml) for full example**):

- `codama = "0.5"` in `[dependencies]` and `[build-dependencies]`
- `borsh`, `solana-program`, `thiserror` in `[dependencies]`
- `solana-pubkey`, `serde_json` in `[build-dependencies]`
- Set `program-id` in `[package.metadata.solana]`

### 2. Create Build Script

Create `build.rs` in your project root. This script runs before compilation to extract program metadata and generate `idl.json`.

**See full implementation:** [build.rs](build.rs)

Key functions:

- Loads program metadata using `Codama::load()`
- Generates JSON IDL with `get_json_idl()`
- Writes formatted IDL to `idl.json`

### 3. Configure Client Generation

Create configuration files for Codama:

**[codama.json](codama.json)** - Specifies IDL path and client generation scripts:

- `js` renderer → generates TypeScript client to `./clients/js/src/generated`
- `rust` renderer → generates Rust client to `./clients/rust`

**[package.json](package.json)** - Node.js dependencies and scripts:

- Scripts: `generate`, `generate:js`, `generate:rust`, `client`
- DevDependencies: `@codama/renderers-js`, `@codama/renderers-rust`, `codama`, `tsx`
- Dependencies: `@solana/kit`

## Codama Macros

### Account State (`CodamaAccount`)

Define account structures with automatic serialization.

**Example:** [src/state.rs](src/state.rs)

```rust
#[derive(CodamaAccount, BorshSerialize, BorshDeserialize, Debug)]
pub struct CounterAccount {
    pub count: u64,
}
```

**What it does:**

- Generates IDL entries for account structure
- Documents field types and sizes
- Creates TypeScript/Rust decoders for reading account data

### Instructions (`CodamaInstructions`)

Define instruction enums with account requirements.

**Example:** [src/instructions.rs](src/instructions.rs)

```rust
#[derive(CodamaInstructions, BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    #[codama(account(name = "counter", signer, writable))]
    #[codama(account(name = "payer", signer, writable))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    InitializeCounter { initial_value: u64 },

    #[codama(account(name = "counter", writable))]
    IncrementCounter,
}
```

**Account Attributes:**

- `name = "..."` - Account name in generated clients
- `signer` - Account must sign the transaction
- `writable` - Account data will be modified
- `default_value = program("system")` - Auto-populate with System Program ID

**Built-in Program Helpers:**

- `program("system")` → `11111111111111111111111111111111`
- `program("token")` → Token Program
- `program("associated-token")` → Associated Token Program
- `program("token-2022")` → Token-2022 Program

### Errors (`CodamaErrors`)

Define custom error types.

**Example:** [src/errors.rs](src/errors.rs)

```rust
#[derive(CodamaErrors, Error, Debug)]
pub enum CounterError {
    #[error("Invalid instruction data provided")]
    InvalidInstructionData,
    #[error("Counter overflow occurred")]
    CounterOverflow,
}
```

**What it does:**

- Documents error codes in IDL
- Generates error messages for clients
- Enables better debugging and error handling

## IDL Generation

The `build.rs` script automatically generates `idl.json` when you build:

```bash
# Build the program (auto-generates IDL)
cargo build-sbf
```

**Generated IDL structure** - See [idl.json](idl.json) for full example

Key structure:

- `kind: "rootNode"` with `standard: "codama"`
- `program` contains instructions, accounts, errors
- Instructions have `arguments` array with discriminator as first argument
- Discriminator has `defaultValueStrategy: "omitted"` so clients auto-include it
- `discriminators` array references which argument field is the discriminator

**Key points:**

- Discriminator is an **argument** with `defaultValueStrategy: "omitted"`
- The `discriminators` array references which field is the discriminator
- `defaultValue` specifies the discriminator value (0, 1, etc.)

## Client Generation

### Generate TypeScript & Rust Clients

```bash
# Install dependencies
pnpm install

# Generate both clients
pnpm generate

# Generate only TypeScript
pnpm generate:js

# Generate only Rust
pnpm generate:rust
```

### Generated Client Structure

**TypeScript:** `clients/js/src/generated/`

```
generated/
├── accounts/          # Account decoders
├── instructions/      # Instruction builders
├── errors/            # Error types
├── programs/          # Program metadata
└── index.ts           # Main exports
```

**Rust:** `clients/rust/`

```
rust/
└── src/
    ├── accounts/
    ├── instructions/
    ├── errors/
    └── lib.rs
```

### Using Generated Clients

**TypeScript Example** - See full implementation: [examples/codama-client.ts](examples/codama-client.ts)

```typescript
import {
  getInitializeCounterInstruction,
  getIncrementCounterInstruction,
} from "./clients/js/src/generated/index.js";

// Create instruction with type-safe builder
const instruction = getInitializeCounterInstruction({
  counter: counterKeypair,
  payer: payerKeypair,
  initialValue: 100n,
  // system_program is auto-populated!
});
```

**Benefits:**

- ✅ **Type-safe** - TypeScript knows all field types
- ✅ **Auto-complete** - IDE suggests available accounts/args
- ✅ **Auto-syncs** - Regenerate after program changes
- ✅ **Less code** - No manual serialization

## Instructions Documentation

### InitializeCounter

Creates a new counter account with an initial value.

**Accounts:**

- `counter` (writable, signer) - The counter account to create
- `payer` (writable, signer) - Account paying for rent
- `system_program` (readonly) - System program (auto-populated)

**Arguments (in IDL):**

- `discriminator: u8` - Instruction discriminator (value: `0`, strategy: `omitted`)
- `initial_value: u64` - Starting counter value

**Serialization (Borsh):**

```
[0, 100, 0, 0, 0, 0, 0, 0, 0]
 ^  ^----- u64 value (100) ----^
 |
 discriminator (u8 = 0)
```

**Note:** The discriminator is an argument in the IDL with `defaultValueStrategy: "omitted"`, which means generated clients automatically include it without the user specifying it.

**Example usage:** See [examples/codama-client.ts](examples/codama-client.ts)

### IncrementCounter

Increments the counter value by 1.

**Accounts:**

- `counter` (writable) - The counter account to increment

**Arguments (in IDL):**

- `discriminator: u8` - Instruction discriminator (value: `1`, strategy: `omitted`)

**Serialization (Borsh):**

```
[1]
 ^
 discriminator (u8 = 1)
```

**Note:** No user-provided arguments needed; the discriminator is automatically included by generated clients.

**Example usage:** See [examples/codama-client.ts](examples/codama-client.ts)

## Development Workflow

### 1. Make Code Changes

```rust
// src/instructions.rs - Add new instruction
#[codama(account(name = "counter", writable))]
DecrementCounter,
```

### 2. Build (Auto-generates IDL)

```bash
cargo build-sbf
```

This runs:

1. `build.rs` extracts metadata from Codama macros
2. Generates/updates `idl.json`
3. Compiles the program to SBF bytecode

### 3. Generate Clients

```bash
pnpm generate
```

Creates updated TypeScript and Rust clients from IDL.

### 4. Test

```bash
# Test Rust code
cargo test

# Test with generated TypeScript client
pnpm client
```

### 5. Deploy & Upload IDL

```bash
# Deploy program
solana program deploy target/deploy/counter_program.so

# Upload IDL for discoverability (devnet, mainnet or you need to load the program metadata program localy)
solana address -k target/deploy/counter_program-keypair.json
npx @solana-program/program-metadata@latest write idl <program-id> ./idl.json
```

## Troubleshooting

### Build Warnings

**`unexpected_cfgs` warnings:**

```
warning: unexpected `cfg` condition name: `target_os`
```

These are harmless warnings from Solana's `entrypoint!` macro. Safe to ignore.

**Debug serialization:**

See [examples/debug_instruction.rs](examples/debug_instruction.rs) for a debug script that shows how Borsh serializes instructions.

```bash
cargo run --example debug_instruction
```

This will print the hex representation, bytes, and length of serialized instructions.

## Learn More

- 📖 [Codama Documentation](https://github.com/codama-idl/codama)
- 📜 [Borsh Specification](https://borsh.io/)
- 🌐 [@solana/kit Documentation](https://www.solanakit.com/docs)
- 🔧 [Codama Visitors](https://github.com/codama-idl/codama/blob/main/packages/visitors/README.md)
