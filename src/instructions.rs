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

    /// CPI: Increment an Anchor counter from this native program
    #[codama(account(name = "anchor_counter", writable))]
    #[codama(account(name = "anchor_authority", signer))]
    #[codama(account(name = "anchor_program"))]
    IncrementAnchorCounter,
}