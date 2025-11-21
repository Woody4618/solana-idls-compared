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

    /// CPI: Increment an Anchor counter using Anchor's generated CPI client
    #[codama(account(name = "anchor_counter", writable))]
    #[codama(account(name = "anchor_authority", signer))]
    #[codama(account(name = "anchor_program"))]
    IncrementAnchorCounter,

    /// CPI: Increment an Anchor counter using manual discriminator construction
    #[codama(account(name = "anchor_counter", writable))]
    #[codama(account(name = "anchor_authority", signer))]
    #[codama(account(name = "anchor_program"))]
    IncrementAnchorCounterRaw,

    /// Self-CPI: Increment the native counter using Codama-style pattern
    #[codama(account(name = "counter", writable))]
    #[codama(account(name = "counter_program"))]
    IncrementCounterSelfCpi,

    /// Self-CPI: Increment using actual Codama-generated CPI client
    #[codama(account(name = "counter", writable))]
    #[codama(account(name = "counter_program"))]
    IncrementCounterCodamaClient,
}
