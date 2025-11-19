mod errors;
mod instructions;
mod processor;
mod state;

pub use errors::*;
pub use instructions::*;
pub use state::*;

use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processor::process(program_id, accounts, instruction_data)
}

#[cfg(test)]
mod test {
    use super::*;
    use borsh::BorshDeserialize;
    use litesvm::LiteSVM;
    use solana_sdk::{
        account::ReadableAccount,
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::Transaction,
    };

    // System Program ID - well-known constant "11111111111111111111111111111111"
    const SYSTEM_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("11111111111111111111111111111111");

    #[test]
    fn test_counter_program() {
        // Create a new LiteSVM instance
        let mut svm = LiteSVM::new();

        // Create a keypair for the transaction payer
        let payer = Keypair::new();

        // Find our program ID (you can get this from your deployed program)
        let program_keypair = read_keypair_file("target/deploy/counter_program-keypair.json")
            .expect("Program keypair file not found");
        let program_id = program_keypair.pubkey();

        // Airdrop some SOL to the payer
        svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

        // Deploy our program
        svm.add_program_from_file(program_id, "target/deploy/counter_program.so")
            .unwrap();

        // Create a keypair for the counter account
        let counter_keypair = Keypair::new();

        // ===== Test 1: Initialize Counter =====

        // Serialize the initialize instruction data
        let initial_value = 100u64;
        let init_instruction_data =
            borsh::to_vec(&CounterInstruction::InitializeCounter { initial_value })
                .expect("Failed to serialize instruction");

        let initialize_instruction = Instruction::new_with_bytes(
            program_id,
            &init_instruction_data,
            vec![
                AccountMeta::new(counter_keypair.pubkey(), true),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ],
        );

        // Create transaction
        let message = Message::new(&[initialize_instruction], Some(&payer.pubkey()));
        let transaction =
            Transaction::new(&[&payer, &counter_keypair], message, svm.latest_blockhash());

        // Send transaction
        let result = svm.send_transaction(transaction);
        assert!(result.is_ok(), "Initialize transaction should succeed");

        // Read the counter account data
        let account = svm
            .get_account(&counter_keypair.pubkey())
            .expect("Account should exist");

        let counter: CounterAccount = CounterAccount::try_from_slice(account.data())
            .expect("Failed to deserialize counter account");

        assert_eq!(
            counter.count, initial_value,
            "Counter should be initialized to 100"
        );

        // ===== Test 2: Increment Counter =====

        // Serialize the increment instruction data
        let increment_instruction_data = borsh::to_vec(&CounterInstruction::IncrementCounter)
            .expect("Failed to serialize instruction");

        let increment_instruction = Instruction::new_with_bytes(
            program_id,
            &increment_instruction_data,
            vec![AccountMeta::new(counter_keypair.pubkey(), true)],
        );

        // Create transaction
        let message = Message::new(&[increment_instruction], Some(&payer.pubkey()));
        let transaction =
            Transaction::new(&[&payer, &counter_keypair], message, svm.latest_blockhash());

        // Send transaction
        let result = svm.send_transaction(transaction);
        assert!(result.is_ok(), "Increment transaction should succeed");

        // Read the updated counter account data
        let account = svm
            .get_account(&counter_keypair.pubkey())
            .expect("Account should exist");

        let counter: CounterAccount = CounterAccount::try_from_slice(account.data())
            .expect("Failed to deserialize counter account");

        assert_eq!(
            counter.count,
            initial_value + 1,
            "Counter should be incremented to 101"
        );

        println!("All tests passed!");
        println!("Initial value: {}", initial_value);
        println!("Final value: {}", counter.count);
    }

    #[test]
    fn test_cpi_to_anchor_counter() {
        // Create a new LiteSVM instance
        let mut svm = LiteSVM::new();

        // Create a keypair for the transaction payer
        let payer = Keypair::new();

        // Load our native program
        let native_program_keypair =
            read_keypair_file("target/deploy/counter_program-keypair.json")
                .expect("Native program keypair file not found");
        let native_program_id = native_program_keypair.pubkey();

        // Load the Anchor program
        let anchor_program_keypair =
            read_keypair_file("anchor-counter/target/deploy/anchor_counter-keypair.json")
                .expect("Anchor program keypair file not found");
        let anchor_program_id = anchor_program_keypair.pubkey();

        // Airdrop some SOL to the payer
        svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

        // Deploy both programs
        svm.add_program_from_file(native_program_id, "target/deploy/counter_program.so")
            .unwrap();
        svm.add_program_from_file(
            anchor_program_id,
            "anchor-counter/target/deploy/anchor_counter.so",
        )
        .unwrap();

        println!("Native Program ID: {}", native_program_id);
        println!("Anchor Program ID: {}", anchor_program_id);

        // Create a keypair for the Anchor counter account
        let anchor_counter_keypair = Keypair::new();

        // ===== Test 1: Initialize Anchor Counter =====
        println!("\n=== Initializing Anchor Counter ===");

        // Anchor's initialize_counter discriminator (from IDL)
        let init_discriminator: [u8; 8] = [67, 89, 100, 87, 231, 172, 35, 124];

        // Serialize the initial value (u64)
        let initial_value = 50u64;
        let mut init_data = init_discriminator.to_vec();
        init_data.extend_from_slice(&initial_value.to_le_bytes());

        let initialize_anchor_instruction = Instruction::new_with_bytes(
            anchor_program_id,
            &init_data,
            vec![
                AccountMeta::new(anchor_counter_keypair.pubkey(), true),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ],
        );

        let message = Message::new(&[initialize_anchor_instruction], Some(&payer.pubkey()));
        let transaction = Transaction::new(
            &[&payer, &anchor_counter_keypair],
            message,
            svm.latest_blockhash(),
        );

        let result = svm.send_transaction(transaction);
        assert!(
            result.is_ok(),
            "Initialize Anchor counter transaction should succeed: {:?}",
            result
        );

        // Read the Anchor counter account data
        let account = svm
            .get_account(&anchor_counter_keypair.pubkey())
            .expect("Anchor counter account should exist");

        // Anchor counter has 8-byte discriminator + 8-byte count + 32-byte authority
        assert!(
            account.data().len() >= 16,
            "Account data too small: {}",
            account.data().len()
        );
        let count = u64::from_le_bytes(account.data()[8..16].try_into().unwrap());
        assert_eq!(count, initial_value, "Anchor counter should be initialized");
        println!("Anchor counter initialized with value: {}", count);

        // ===== Test 2: CPI from Native to Anchor =====
        println!("\n=== Testing CPI: Native → Anchor ===");

        // Serialize the CPI instruction (variant 2 = IncrementAnchorCounter)
        let cpi_instruction_data = borsh::to_vec(&CounterInstruction::IncrementAnchorCounter)
            .expect("Failed to serialize instruction");

        let cpi_instruction = Instruction::new_with_bytes(
            native_program_id,
            &cpi_instruction_data,
            vec![
                AccountMeta::new(anchor_counter_keypair.pubkey(), false),
                AccountMeta::new_readonly(payer.pubkey(), true),
                AccountMeta::new_readonly(anchor_program_id, false),
            ],
        );

        let message = Message::new(&[cpi_instruction], Some(&payer.pubkey()));
        let transaction = Transaction::new(&[&payer], message, svm.latest_blockhash());

        let result = svm.send_transaction(transaction);
        assert!(
            result.is_ok(),
            "CPI to Anchor counter should succeed: {:?}",
            result
        );

        // Read the updated Anchor counter account data
        let account = svm
            .get_account(&anchor_counter_keypair.pubkey())
            .expect("Anchor counter account should exist");

        let count = u64::from_le_bytes(account.data()[8..16].try_into().unwrap());
        assert_eq!(
            count,
            initial_value + 1,
            "Anchor counter should be incremented via CPI"
        );

        println!("✅ CPI Test Passed!");
        println!("Initial value: {}", initial_value);
        println!("Final value after CPI: {}", count);
    }
}
