# Solana IDL Approaches Compared

A comprehensive comparison of three approaches to building Solana programs: **Native Solana**, **Native + Codama**, and **Anchor Framework**.

This repository contains a simple counter program implemented in multiple ways to demonstrate the differences in developer experience, tooling, and generated artifacts.

## 🎯 Project Goals

- Compare developer experience across approaches
- Demonstrate IDL generation strategies
- Show client code generation differences
- Highlight trade-offs in control vs. convenience

## 📁 Repository Structure

```
solana-idls-compared/
│
├── src/                          # Native Solana + Codama
│   ├── lib.rs                   # Program entrypoint
│   ├── state.rs                 # Account definitions (with CodamaAccount)
│   ├── instructions.rs          # Instructions (with CodamaInstructions)
│   ├── processor.rs             # Business logic
│   └── errors.rs                # Errors (with CodamaErrors)
│
├── anchor-counter/              # Anchor Framework
│   ├── programs/
│   │   └── anchor-counter/
│   │       └── src/lib.rs      # Anchor program (all-in-one)
│   └── tests/                   # TypeScript tests
│
├── examples/
│   ├── client.rs               # Manual Rust client (native)
│   └── codama-client.ts        # Generated TypeScript client (Codama)
│
├── clients/                     # Auto-generated from IDL
│   ├── js/                     # TypeScript client (Codama)
│   └── rust/                   # Rust client (Codama)
│
├── build.rs                    # IDL generation (Codama)
├── idl.json                    # Generated IDL (Codama)
├── codama.json                 # Client generation config
└── package.json                # Node.js scripts
```

## 🔄 The Counter Program

All implementations provide the same functionality:

### Initialize Counter

Creates a new counter account with an initial value.

- **Accounts:** counter (writable, signer), payer (writable, signer), system_program
- **Args:** `initial_value: u64`

### Increment Counter

Increments the counter by 1.

- **Accounts:** counter (writable)
- **Args:** None

## 📊 Approach Comparison

### Overview

| Approach            | IDL Generation      | Client Generation      | PDA Support        | Serialization    | Boilerplate | Control | Best For               |
| ------------------- | ------------------- | ---------------------- | ------------------ | ---------------- | ----------- | ------- | ---------------------- |
| **Native Solana**   | Manual              | Manual                 | Manual             | Any (custom)     | High        | Maximum | Low-level optimization |
| **Native + Codama** | Auto (build.rs)     | Auto (TypeScript/Rust) | Manual/Visitor API | Borsh, custom    | Medium      | High    | Custom logic + tooling |
| **Anchor**          | Auto (anchor build) | Auto (TypeScript)      | Auto-resolution    | Borsh, Zero Copy | Low         | Medium  | Rapid development      |

### Detailed Comparison

#### 1. Code Structure

**Native Solana:**

```rust
// Manual everything
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Manual deserialization
    // Manual account validation
    // Manual instruction routing
}
```

**Native + Codama:**

```rust
// Annotated structs/enums
#[derive(CodamaInstructions, BorshSerialize, BorshDeserialize)]
pub enum CounterInstruction {
    #[codama(account(name = "counter", signer, writable))]
    #[codama(account(name = "payer", signer, writable))]
    InitializeCounter { initial_value: u64 },
}

// Manual processing, auto IDL
```

**Anchor:**

```rust
// Declarative, minimal boilerplate
#[program]
pub mod counter {
    pub fn initialize_counter(ctx: Context<Initialize>, initial_value: u64) -> Result<()> {
        // Auto account validation
        // Auto deserialization
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 8)]
    pub counter: Account<'info, Counter>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}
```

#### 2. IDL Generation

**Native Solana:**

- ❌ No IDL
- Manual documentation
- No standard format

**Native + Codama:**

- ✅ Automatic via `build.rs`
- Generated on every build
- Standard Codama IDL format
- Requires derive macros
- Upload via Program Metadata program

**Anchor:**

- ✅ Automatic via `anchor build`
- Standard Anchor IDL format
- Generated from program structure
- No extra macros needed
- Upload automatic when program deploys

#### 3. Client Generation

**Native Solana:**

```typescript
// Manual instruction building
const instruction = new TransactionInstruction({
  keys: [
    { pubkey: counter, isSigner: true, isWritable: true },
    { pubkey: payer, isSigner: true, isWritable: true },
    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
  ],
  programId: PROGRAM_ID,
  data: Buffer.from([0, ...initial_value_bytes]), // Manual serialization
});
```

**Native + Codama:**

```typescript
// Generated type-safe builders
import { getInitializeCounterInstruction } from "./generated";

const instruction = getInitializeCounterInstruction({
  counter: counterKeypair,
  payer: payerKeypair,
  initialValue: 100n,
  // system_program auto-populated!
});
```

**Anchor:**

Most accounts and PDAs have auto resolution from the IDL.

```typescript
// Generated Anchor client
import { Program } from "@coral-xyz/anchor";

await program.methods
  .initializeCounter(new BN(100))
  .accounts({
    counter: counterKeypair.publicKey,
    payer: payer.publicKey,
  })
  .signers([counterKeypair])
  .rpc();
```

#### 4. Account Validation

**Native Solana:**

```rust
// Manual validation
let accounts_iter = &mut accounts.iter();
let counter_account = next_account_info(accounts_iter)?;
let payer_account = next_account_info(accounts_iter)?;

if !counter_account.is_signer {
    return Err(ProgramError::MissingRequiredSignature);
}
if !counter_account.is_writable {
    return Err(ProgramError::InvalidAccountData);
}
// ... more manual checks
```

**Native + Codama:**

```rust
// Still manual validation (Codama only helps with IDL/clients)
let accounts_iter = &mut accounts.iter();
let counter_account = next_account_info(accounts_iter)?;
// ... same as native
```

**Anchor:**

Anchor is more secure since it does lots of security checks for you automatically.

```rust
// Automatic validation via #[derive(Accounts)]
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = payer, space = 8 + 8)]
    pub counter: Account<'info, Counter>,  // Auto-validated
    #[account(mut)]
    pub payer: Signer<'info>,  // Auto checks is_signer
    pub system_program: Program<'info, System>,  // Auto checks program_id
}
```

#### 5. Instruction Discriminators

**Native Solana:**

```rust
// Custom discriminator (you choose)
match instruction_data[0] {
    0 => process_initialize(program_id, accounts, &instruction_data[1..]),
    1 => process_increment(program_id, accounts),
    _ => Err(ProgramError::InvalidInstructionData),
}
```

**Native + Codama:**

```rust
// Borsh enum discriminator (1 byte for variants 0-127)
// Variant 0 => [0]
// Variant 1 => [1]
pub enum CounterInstruction {
    InitializeCounter { initial_value: u64 },  // discriminator: 0
    IncrementCounter,                           // discriminator: 1
}
```

**Anchor:**

```rust
// 8-byte discriminator derived from instruction name
// "initialize_counter" => [175, 175, 109, 31, 13, 152, 155, 237]
// "increment_counter"  => [11, 18, 104, 9, 104, 174, 59, 33]
#[program]
pub mod counter {
    pub fn initialize_counter(...) -> Result<()> { ... }
    pub fn increment_counter(...) -> Result<()> { ... }
}
```

#### 6. Testing

**Native Solana:**

```rust
// Unit tests with solana_program_test or litesvm
#[cfg(test)]
mod tests {
    use litesvm::LiteSVM;

    #[test]
    fn test_initialize() {
        let mut svm = LiteSVM::new();
        // Manual instruction construction
        // Manual transaction building
        // Manual result verification
    }
}
```

**Native + Codama:**

```rust
// Same as native (Codama doesn't affect testing)
// But can also test with generated TypeScript client
```

```typescript
// TypeScript with generated client
import { getInitializeCounterInstruction } from "./generated";

const instruction = getInitializeCounterInstruction({
  counter,
  payer,
  initialValue: 100n,
});
// Send transaction...
```

**Anchor:**

```typescript
// Built-in testing framework
import { Program } from '@coral-xyz/anchor';

describe("counter", () => {
  it("Initializes counter", async () => {
    await program.methods
      .initializeCounter(new BN(100))
      .accounts({ ... })
      .rpc();

    const account = await program.account.counter.fetch(counter);
    assert.equal(account.count.toNumber(), 100);
  });
});
```

#### 7. PDA (Program Derived Address) Handling

**Native Solana:**

```rust
// Manual PDA derivation
let (pda, bump) = Pubkey::find_program_address(
    &[b"counter", user.key.as_ref()],
    program_id
);

// Manual validation
if pda != *counter_account.key {
    return Err(ProgramError::InvalidSeeds);
}
```

**Native + Codama:**

```rust
// Manual derivation in program, but can document in IDL
let (pda, bump) = Pubkey::find_program_address(
    &[b"counter", user.key.as_ref()],
    program_id
);

// Can add PDA metadata using addPdasVisitor in codama.json
```

```typescript
// Client can use PDA info from IDL if added via visitors
import { findCounterPda } from "./generated";
const [pda] = findCounterPda({ user: userPublicKey });
```

**Anchor:**

```rust
// Automatic PDA derivation and validation
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + 8,
        seeds = [b"counter", user.key().as_ref()],
        bump
    )]
    pub counter: Account<'info, Counter>,
}
```

```typescript
// Client automatically resolves PDAs from seeds in IDL
const [counterPda] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("counter"), user.publicKey.toBuffer()],
  program.programId
);

await program.methods
  .initialize(new BN(100))
  .accounts({
    counter: counterPda, // Anchor validates this matches seeds and you dont event have to pass it into the function. Its auto resolved for you!
    payer: payer.publicKey,
  })
  .rpc();
```

**Key Differences:**

- ✅ **Anchor**: Automatic PDA derivation and validation with `seeds` and `bump` constraints. The account discriminator and seeds are included in IDL, enabling client-side auto-resolution.
- ⚙️ **Codama**: Manual PDA derivation in program. Can add PDA metadata to IDL using `addPdasVisitor` for client generation.
- 🔧 **Native**: Completely manual - write all derivation and validation logic yourself.

#### 8. Serialization Options

**Native Solana:**

Supports any serialization format:

```rust
// Borsh
use borsh::{BorshDeserialize, BorshSerialize};

// Bincode (used by system programs)
use bincode;

// Custom binary format
pub fn custom_deserialize(data: &[u8]) -> Result<MyStruct, ProgramError> {
    // Hand-written deserialization for maximum efficiency
}
```

**Native + Codama:**

Flexible serialization support:

- ✅ **Borsh** (primary, default)
- ✅ **Custom formats**
- ✅ **Mixed formats** (different instructions can use different formats)

```rust
// Can use Borsh for most data
#[derive(CodamaAccount, BorshSerialize, BorshDeserialize)]
pub struct CounterAccount {
    pub count: u64,
}

// But document alternative formats in IDL if needed
// (e.g., for specific instructions requiring JSON)
```

**Anchor:**

Primarily Borsh:

- ✅ **Borsh** (primary, optimized)
- Zero Copy
- Works well for most use cases

```rust
#[account]
pub struct Counter {
    pub count: u64,  // Automatically uses Borsh
}
```

**Key Differences:**

- 🎨 **Codama**: Most flexible - supports Borsh and custom formats. Can mix serialization strategies.
- 📦 **Anchor**: Standardized on Borsh for consistency and optimization. Simple and reliable. Also Supports Zero Copy.
- 🔧 **Native**: Complete freedom but requires manual implementation of all serialization logic.

#### 9. Program Size

| Approach        | Compiled Size | Note                          |
| --------------- | ------------- | ----------------------------- |
| Native Solana   | ~Small        | Minimal dependencies          |
| Native + Codama | ~Small        | Codama only affects build     |
| Anchor          | ~Larger       | Additional framework overhead |

\_Note: Actual sizes depend on program complexity.

#### 10. Learning Curve

**Native Solana:**

- ⛰️ **Steep** - Must understand low-level Solana concepts
- Manual serialization/deserialization (Borsh or other)
- Manual account validation
- Manual error handling
- No safety nets

**Native + Codama:**

- 🏔️ **Medium-High** - Same as native for program logic
- Additional: Learn Codama macros
- Additional: Configure build.rs
- Benefits: Auto-generated clients reduce client-side complexity

**Anchor:**

- ⛰️ **Gentler** - Abstracts low-level details
- Declarative account validation
- Built-in serialization
- Helpful error messages
- Faster to get started

## 🚀 Getting Started

### Prerequisites

- [Solana install script for all deps](https://solana.com/de/docs/intro/installation)

### Native + Codama Program

```bash
# Build program (auto-generates IDL)
cargo build-sbf

# Generate clients
pnpm install
pnpm generate

# Deploy
solana program deploy target/deploy/counter_program.so

solana address -k target/deploy/counter_program-keypair.json
npx @solana-program/program-metadata@latest write idl <program-id> ./idl.json

# Test with TypeScript client
pnpm client

# Test with Rust client
cargo run --example client
```

### Anchor Program

```bash
cd anchor-counter

# Build (generates IDL + program)
anchor build

# Test
npm install
npm test

# Deploy auto uploads the IDL
anchor deploy
```

## 💻 Example Clients

### Manual Rust Client (Native)

```bash
cargo run --example client
```

**Features:**

- Manual instruction construction
- Direct Borsh serialization
- Uses `solana-client` and `solana-sdk`

### Codama TypeScript Client

```bash
# Start validator
solana-test-validator

# Run client
pnpm client
```

**Features:**

- Auto-generated instruction builders
- Type-safe with TypeScript
- Uses `@solana/kit`

### Anchor TypeScript Client

```bash
cd anchor-counter
npm test
```

**Features:**

- Anchor Program integration
- Built-in test framework
- Uses `@coral-xyz/anchor`

## 🎯 Which Approach Should You Use?

### Choose **Native Solana** if:

- ✅ You need maximum control over every byte
- ✅ You're building ultra-optimized programs
- ✅ You want to learn Solana internals deeply
- ✅ You're building system-level programs

### Choose **Native + Codama** if:

- ✅ You want control over program logic
- ✅ You need custom instruction discriminators
- ✅ You need **flexible serialization** (Borsh, JSON, or custom formats)
- ✅ You want auto-generated clients in multiple languages
- ✅ You're comfortable with lower-level Solana and manual PDA handling

### Choose **Anchor** if:

- ✅ You want to ship fast
- ✅ You want to write a secure program
- ✅ You need **automatic PDA derivation and validation**
- ✅ You want strong safety guarantees with account constraints
- ✅ You're new to Solana development
- ✅ You prefer standardized Borsh serialization

## 📚 Learn More

### Documentation

- **Native Solana:** [Program Structure](https://solana.com/docs/programs/rust/program-structure/)
- **Codama:** [GitHub](https://github.com/codama-idl/codama) | [Detailed Guide](./CODAMA.md)
- **Anchor:** [Official Docs](https://www.anchor-lang.com/) | [Book](https://book.anchor-lang.com/)

### Tools

- **@solana/kit:** [Website](https://www.solanakit.com/docs)
- **Borsh:** [Specification](https://borsh.io/)
- **Solana Program Library:** [GitHub](https://github.com/solana-program)

### Community

- [Solana Stack Exchange](https://solana.stackexchange.com/)
- [Anchor Discord](https://discord.gg/anchor)
- [Solana Developer Discord](https://discord.gg/solana)

## 🤝 Contributing

Contributions welcome! Feel free to:

- Add new example implementations
- Improve documentation
- Fix bugs or issues
- Share feedback

## 📄 License

MIT

---

**Questions or feedback?** Open an issue or reach out on Discord!
