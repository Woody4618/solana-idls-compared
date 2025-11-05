use borsh::{BorshDeserialize, BorshSerialize};
use codama::CodamaInstructions;

#[derive(CodamaInstructions, BorshSerialize, BorshDeserialize, Debug)]
pub enum CounterInstruction {
    #[codama(account(name = "counter", signer, writable))]
    #[codama(account(name = "payer", signer, writable))]
    #[codama(account(name = "system_program", default_value = program("system")))]
    InitializeCounter { initial_value: u64 },

    #[codama(account(name = "counter", writable))]
    IncrementCounter,
}