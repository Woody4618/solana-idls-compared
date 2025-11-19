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
│   ├── native-client.rs        # Manual Rust client (native)
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

| Approach            | IDL Generation      | Client Generation      | PDA Support           | CPI Support             | Serialization    | Boilerplate | Control | Best For               |
| ------------------- | ------------------- | ---------------------- | --------------------- | ----------------------- | ---------------- | ----------- | ------- | ---------------------- |
| **Native Solana**   | Manual              | Manual                 | Manual                | Manual                  | Any (custom)     | High        | Maximum | Low-level optimization |
| **Native + Codama** | Auto (build.rs)     | Auto (TypeScript/Rust) | Manual (Visitor API?) | Manual                  | Borsh, custom    | Medium      | High    | Custom logic + tooling |
| **Anchor**          | Auto (anchor build) | Auto (TypeScript)      | Auto-resolution       | Auto (Anchor-to-Anchor) | Borsh, Zero Copy | Low         | Medium  | Rapid development      |

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
// "initialize_counter" => [67, 89, 100, 87, 231, 172, 35, 124]
// "increment_counter"  => [16, 125, 2, 171, 73, 24, 207, 229]
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
// Manual derivation in program
let (pda, bump) = Pubkey::find_program_address(
    &[b"counter", user.key.as_ref()],
    program_id
);

// Manual validation still required
if pda != *counter_account.key {
    return Err(ProgramError::InvalidSeeds);
}
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
- ⚙️ **Codama**: Manual PDA derivation and validation in program.
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
    // Hand-written deserialization for custom data (only if really needed since it decreases composability)
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

#### 11. Cross-Program Invocation (CPI)

CPIs allow one program to call another program's instructions. This repository includes examples of:

- **Native → Anchor CPI**: Native program calling Anchor program
- **Anchor → Native CPI**: Anchor program calling native program

**Native Solana CPI:**

```rust
// Manual CPI construction
use solana_program::instruction::Instruction;

// Build instruction manually with correct discriminator
let discriminator: [u8; 8] = [16, 125, 2, 171, 73, 24, 207, 229]; // Anchor's increment_counter
let instruction_data = discriminator.to_vec();

let cpi_instruction = Instruction {
    program_id: *target_program_id,
    accounts: vec![
        AccountMeta::new(*counter_account, false),
        AccountMeta::new_readonly(*authority, true),
    ],
    data: instruction_data,
};

// Invoke the CPI
invoke(&cpi_instruction, &[counter_account, authority, target_program])?;
```

**Native + Codama CPI:**

Same as native Solana - Codama helps with client generation but doesn't simplify CPIs:

```rust
// Still manual CPI construction
let discriminator: [u8; 8] = [16, 125, 2, 171, 73, 24, 207, 229];
let cpi_instruction = Instruction {
    program_id: *anchor_program.key,
    accounts: vec![
        AccountMeta::new(*anchor_counter_account.key, false),
        AccountMeta::new_readonly(*anchor_authority_account.key, true),
    ],
    data: discriminator.to_vec(),
};

invoke(&cpi_instruction, &[anchor_counter_account, anchor_authority_account, anchor_program])?;
```

**Anchor CPI:**

Anchor can generate CPI helper functions, but for calling native programs you still need manual construction:

```rust
// Use anchor_lang's re-exports (add to top of file)
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke;

// Calling a native program from Anchor - still manual
let instruction_data: Vec<u8> = vec![1]; // Native enum variant

let cpi_instruction = Instruction {
    program_id: ctx.accounts.native_program.key(),
    accounts: vec![
        AccountMeta::new(ctx.accounts.native_counter.key(), false),
    ],
    data: instruction_data,
};

invoke(
    &cpi_instruction,
    &[ctx.accounts.native_counter.to_account_info()],
)?;
```

**For Anchor-to-Anchor CPIs**, Anchor provides automatic CPI helper generation:

```rust
// Anchor-to-Anchor CPI (much easier with generated helpers)
use other_program::cpi;

cpi::increment_counter(
    CpiContext::new(
        ctx.accounts.other_program.to_account_info(),
        cpi::accounts::IncrementCounter {
            counter: ctx.accounts.counter.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    ),
)?;
```

**Key Differences:**

| Aspect               | Native/Codama                     | Anchor                                          |
| -------------------- | --------------------------------- | ----------------------------------------------- |
| **CPI Construction** | Always manual                     | Manual for native, auto-generated for Anchor    |
| **Discriminators**   | Must know target format           | Auto-handled for Anchor-to-Anchor               |
| **Account Passing**  | Manual AccountMeta construction   | Type-safe with CpiContext for Anchor-to-Anchor  |
| **Type Safety**      | None - raw bytes                  | Full type safety for Anchor-to-Anchor           |
| **Error Handling**   | Manual error propagation          | Automatic error mapping for Anchor-to-Anchor    |
| **Best For**         | Maximum control, any program type | Rapid development when both programs are Anchor |

**Testing CPIs:**

Both approaches include tests demonstrating CPIs:

- **Native Test**: `cargo test test_cpi_to_anchor_counter` (in `src/lib.rs`)
- **Anchor Test**: `anchor test` (includes CPI to native program test)

See the test files for complete working examples of both CPI directions!

📖 **Detailed CPI Documentation**: See [CPI_EXAMPLES.md](./CPI_EXAMPLES.md) for in-depth explanations, implementation details, and how to find instruction discriminators.

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
cargo run --example native-client

# Run Rust tests (including CPI tests)
# Note: Build both programs first
cd anchor-counter && anchor build && cd ..
cargo build-sbf
cargo test
```

### Anchor Program

```bash
cd anchor-counter

# Build (generates IDL + program)
anchor build

# Test (includes CPI tests)
# Note: Build native program first for CPI test
cd .. && cargo build-sbf && cd anchor-counter
npm install
anchor test

# Deploy auto uploads the IDL
anchor deploy
```

## 💻 Example Clients

### Manual Rust Client (Native)

```bash
cargo run --example native-client
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
anchor test
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

For most use cases its recommended to use Anchor since it provides a lot of safety guarantees and is easier to get started with.

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

Quick Summary of what is missing and what we could do:

Codama missing / todos

- Codama does not create package.json and cargo.toml for clients
- Codama needs a framework around it to make build, generate and upload IDL automatically. Could be done in anchor maybe.
- Kit compatible version for LiteSVM missing. So you need to use rust, but then in the client you likely use ts.
- Wallet connect button for kit missing (Does https://docs.hermis.io/ will that gap or maybe what gui is building?)
- Codama clients are missing the option to call them on the fly like anchor programs.
  - Currently cant connect anchor programs using kit
- Codama does not support has one constraint so we cant use it in anchor programs
- Codama does need static program ID and fails conversion:
  - Source: https://solana.stackexchange.com/questions/22836/codamaerror-program-id-kind-account-is-not-implemented?utm_source=chatgpt.com
- Automatic accounts resolution for PDAs?
- Are errors mapped the same way?
- Events: Anchor events are emitted via logs. Will that be part of the kit client as well?
- Support enums, lists, generics, etc.

- Missing template or anchor init for anchor/codama kit clients.
  - Currently cant connect anchor programs using kit
  - We could let Hoodies build that and contribute it to codama repo.
- Codama does not support hasOne constraint
- Account types? If any accounts/instructions use edge types (e.g., u128/i128, nested enums, fixed-length arrays, Vec<Vec<u8>>, optional complex enums), verify Codama’s generator matches your on-chain layout exactly. Mismatches here are subtle and painful.
- Errors: Anchor errors export fine, but ensure your Kit client maps them to nice TS discriminants.
- Events: Anchor events are emitted via logs. Make sure your Kit stack has the log parser wired (and that your transport supports it).
- HasOne and seeds are missing from the IDL in anchor already. So the ts client is generated from the rust source code metadat and macros. Thats why it can be added.

- Create anchor init or template that uses the anchor idl to generate codama to generate kit client
- Implement codama kit client generation into anchor to generate a rust and kit client next to the type script client

- Talked to Brian about Vixxen to implement IDL parsing in grpc
- IDL parsed data will also become much easier

Notes:

- Enums, Lists, etc
- Should we add generics?

Lets discuss with Loris next week

Video 1
Show how to use codama with anchor
How to call the program using the codama client
Show the kit compatible version

Video 2
native program how generate Codama client for that
How to call the programs using the codama clients
Cant get a web3js client out of codama
