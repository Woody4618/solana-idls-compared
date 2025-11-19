use crate::{instructions::CounterInstruction, state::CounterAccount};
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

/// Perform a CPI to increment an Anchor counter
/// This demonstrates how to call an Anchor program from a native Solana program
fn process_increment_anchor_counter(accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let anchor_counter_account = next_account_info(accounts_iter)?;
    let anchor_authority_account = next_account_info(accounts_iter)?;
    let anchor_program = next_account_info(accounts_iter)?;

    msg!("Performing CPI to Anchor program...");

    // Anchor's increment_counter instruction discriminator (from IDL)
    // This is derived from: anchor_lang::prelude::hash::hash(b"global:increment_counter")
    // For Anchor, the discriminator is the first 8 bytes of the SHA256 hash
    // of the namespace:instruction_name string
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

    msg!("Successfully incremented Anchor counter via CPI");
    Ok(())
}
