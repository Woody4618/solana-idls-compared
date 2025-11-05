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
    use borsh::{BorshDeserialize};
    use litesvm::LiteSVM;
    use solana_sdk::{
        account::ReadableAccount,
        instruction::{AccountMeta, Instruction},
        message::Message,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
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
        let program_id = Pubkey::from_str_const("ATjcKTRrFZwdTjSYpheKkEKKAPzf4iUoK6ZtPqJysnyN");

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
}
