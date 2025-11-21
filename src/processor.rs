use crate::{instructions::CounterInstruction, state::CounterAccount};
use anchor_lang::ToAccountInfo; // Required for Anchor CPI client
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction,
    sysvar::{rent::Rent, Sysvar},
};

pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Unpack instruction data
    let instruction = CounterInstruction::try_from_slice(instruction_data)
        .map_err(|_| ProgramError::InvalidInstructionData)?;

    // Match instruction type
    match instruction {
        CounterInstruction::InitializeCounter { initial_value } => {
            process_initialize_counter(program_id, accounts, initial_value)?
        }
        CounterInstruction::IncrementCounter => process_increment_counter(program_id, accounts)?,
        CounterInstruction::IncrementAnchorCounter => process_increment_anchor_counter(accounts)?,
        CounterInstruction::IncrementAnchorCounterRaw => {
            process_increment_anchor_counter_raw(accounts)?
        }
        CounterInstruction::IncrementCounterSelfCpi => {
            process_increment_counter_self_cpi(program_id, accounts)?
        }
        CounterInstruction::IncrementCounterCodamaClient => {
            process_increment_counter_codama_client(program_id, accounts)?
        }
    };
    Ok(())
}

// Initialize a new counter account
fn process_initialize_counter(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    initial_value: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let counter_account = next_account_info(accounts_iter)?;
    let payer_account = next_account_info(accounts_iter)?;
    let system_program = next_account_info(accounts_iter)?;

    // Size of our counter account
    let account_space = 8; // u64 requires 8 bytes

    // Calculate minimum balance for rent exemption
    let rent = Rent::get()?;
    let required_lamports = rent.minimum_balance(account_space);

    // Create the counter account
    invoke(
        &system_instruction::create_account(
            payer_account.key,    // Account paying for the new account
            counter_account.key,  // Account to be created
            required_lamports,    // Amount of lamports to transfer to the new account
            account_space as u64, // Size in bytes to allocate for the data field
            program_id,           // Set program owner to our program
        ),
        &[
            payer_account.clone(),
            counter_account.clone(),
            system_program.clone(),
        ],
    )?;

    // Create a new CounterAccount struct with the initial value
    let counter_data = CounterAccount {
        count: initial_value,
    };

    // Get a mutable reference to the counter account's data
    let mut account_data = &mut counter_account.data.borrow_mut()[..];

    // Serialize the CounterAccount struct into the account's data
    counter_data.serialize(&mut account_data)?;

    msg!("Counter initialized with value: {}", initial_value);

    Ok(())
}

// Update an existing counter's value
fn process_increment_counter(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;

    // Verify account ownership
    if counter_account.owner != program_id {
        return Err(ProgramError::IncorrectProgramId);
    }

    // Mutable borrow the account data
    let mut data = counter_account.data.borrow_mut();

    // Deserialize the account data into our CounterAccount struct
    let mut counter_data: CounterAccount = CounterAccount::try_from_slice(&data)?;

    // Increment the counter value
    counter_data.count = counter_data
        .count
        .checked_add(1)
        .ok_or(ProgramError::InvalidAccountData)?;

    // Serialize the updated counter data back into the account
    counter_data.serialize(&mut &mut data[..])?;

    msg!("Counter incremented to: {}", counter_data.count);
    Ok(())
}

/// Perform a CPI to increment an Anchor counter using Anchor's generated CPI client
/// This demonstrates how to call an Anchor program from a native Solana program with type safety
///
/// Benefits of using Anchor's CPI client:
/// - Type-safe: Compile-time validation of accounts and parameters
/// - Auto-generated: No manual discriminator construction needed
/// - Maintainable: Automatically updates when Anchor program changes
/// - Error-resistant: Can't pass wrong accounts or wrong order
fn process_increment_anchor_counter(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let anchor_counter_account = next_account_info(accounts_iter)?;
    let anchor_authority_account = next_account_info(accounts_iter)?;
    let anchor_program = next_account_info(accounts_iter)?;

    msg!("Performing CPI to Anchor program using generated CPI client...");

    // ✅ Use Anchor's type-safe CPI client
    // This is auto-generated from the anchor-counter program when the 'cpi' feature is enabled
    let cpi_program = anchor_program.to_account_info();
    let cpi_accounts = anchor_counter::cpi::accounts::IncrementCounter {
        counter: anchor_counter_account.to_account_info(),
        authority: anchor_authority_account.to_account_info(),
    };
    let cpi_ctx = anchor_lang::context::CpiContext::new(cpi_program, cpi_accounts);
    anchor_counter::cpi::increment_counter(cpi_ctx)?;

    msg!("Successfully incremented Anchor counter via CPI (type-safe client)");
    Ok(())
}

/// Perform a CPI to increment an Anchor counter using manual discriminator construction
/// This demonstrates the low-level approach without using Anchor's generated CPI client
///
/// This approach is useful when:
/// - You don't have access to the Anchor program's crate
/// - You want to minimize dependencies
/// - You need to understand the raw instruction format
/// - You're calling from a non-Anchor program
///
/// Drawbacks compared to the CPI client:
/// - Manual: Must look up discriminators from the IDL
/// - Error-prone: No compile-time validation of accounts
/// - Maintenance: Must update manually when the Anchor program changes
/// - No type safety: Easy to pass wrong accounts or wrong order
fn process_increment_anchor_counter_raw(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let anchor_counter_account = next_account_info(accounts_iter)?;
    let anchor_authority_account = next_account_info(accounts_iter)?;
    let anchor_program = next_account_info(accounts_iter)?;

    msg!("Performing CPI to Anchor program using manual discriminator...");

    // Anchor's increment_counter instruction discriminator (from IDL)
    // This is derived from: anchor_lang::prelude::hash::hash(b"global:increment_counter")
    // For Anchor, the discriminator is the first 8 bytes of the SHA256 hash
    // of the namespace:instruction_name string
    // You can find this in: anchor-counter/target/idl/anchor_counter.json
    let discriminator: [u8; 8] = [16, 125, 2, 171, 73, 24, 207, 229];

    // Build the instruction data (just the discriminator, no additional args)
    let instruction_data = discriminator.to_vec();

    // Create the instruction for the CPI
    let cpi_instruction = solana_program::instruction::Instruction {
        program_id: *anchor_program.key,
        accounts: vec![
            solana_program::instruction::AccountMeta::new(*anchor_counter_account.key, false),
            solana_program::instruction::AccountMeta::new_readonly(
                *anchor_authority_account.key,
                true,
            ),
        ],
        data: instruction_data,
    };

    // Invoke the CPI
    invoke(
        &cpi_instruction,
        &[
            anchor_counter_account.clone(),
            anchor_authority_account.clone(),
            anchor_program.clone(),
        ],
    )?;

    msg!("Successfully incremented Anchor counter via CPI (raw/manual approach)");
    Ok(())
}

/// Perform a self-CPI to increment the counter using Codama-style CPI pattern
/// This demonstrates how to use a Codama-generated approach for calling your own program
///
/// The Codama-generated client (in clients/rust/) provides CPI helpers similar to Anchor:
/// - IncrementCounterCpi and IncrementCounterCpiBuilder
/// - Type-safe account validation
/// - Builder pattern for clarity
///
/// However, due to version compatibility issues with mixing crate names, we implement
/// the CPI pattern directly here, inspired by Codama's generated code structure.
///
/// Benefits of this approach:
/// - Type-safe: Uses known instruction discriminators
/// - Clean: Builder-like pattern for clarity
/// - Compatible: Works within solana-program ecosystem
///
/// Use cases for self-CPI:
/// - Program upgradability: Old version calls new version
/// - Composability: Break complex logic into smaller instructions
/// - Testing: Verify instruction behavior in different contexts
fn process_increment_counter_self_cpi(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let counter_program = next_account_info(accounts_iter)?;

    // Verify we're calling our own program
    if counter_program.key != program_id {
        msg!("Error: Program ID mismatch");
        return Err(ProgramError::IncorrectProgramId);
    }

    msg!("Performing self-CPI using Codama-style pattern...");

    // ✅ Use Codama-style CPI pattern (inspired by generated code)
    // Discriminator for IncrementCounter (variant 1 in the enum)
    const INCREMENT_COUNTER_DISCRIMINATOR: u8 = 1;

    // Build instruction data (just the discriminator)
    let instruction_data = vec![INCREMENT_COUNTER_DISCRIMINATOR];

    // Create the CPI instruction
    use solana_program::instruction::{AccountMeta, Instruction};
    let cpi_instruction = Instruction {
        program_id: *program_id,
        accounts: vec![AccountMeta::new(*counter_account.key, false)],
        data: instruction_data,
    };

    // Invoke the CPI
    invoke(&cpi_instruction, &[counter_account.clone()])?;

    msg!("Successfully incremented counter via self-CPI (Codama-style pattern)");
    Ok(())
}

/// Perform a self-CPI using the actual Codama-generated CPI client
/// This demonstrates using Codama's auto-generated CPI helpers directly
///
/// The Codama-generated client (in clients/rust/instructions/) provides:
/// - IncrementCounterCpi - Direct CPI struct
/// - IncrementCounterCpiBuilder - Builder pattern for constructing CPIs
///
/// Benefits of using Codama's generated client:
/// - Type-safe account structures
/// - Builder pattern for clarity
/// - Auto-generated from IDL
/// - Consistent with Anchor's CPI style
/// - Handles instruction data serialization automatically
fn process_increment_counter_codama_client(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let counter_account = next_account_info(accounts_iter)?;
    let counter_program = next_account_info(accounts_iter)?;

    // Verify we're calling our own program
    if counter_program.key != program_id {
        msg!("Error: Program ID mismatch");
        return Err(ProgramError::IncorrectProgramId);
    }

    crate::codama_client::instructions::IncrementCounterCpiBuilder::new(counter_program)
        .counter(counter_account)
        .invoke()?;

    Ok(())
}
