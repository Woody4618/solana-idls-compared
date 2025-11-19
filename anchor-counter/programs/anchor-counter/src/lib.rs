use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke;

declare_id!("27YreJqker2o5TvzzLUsiC9ZGMdPThvEm8qZBNDw5EWX");

#[program]
pub mod anchor_counter {
    use super::*;

    /// Initialize a new counter account with an initial value
    pub fn initialize_counter(ctx: Context<InitializeCounter>, initial_value: u64) -> Result<()> {
        let counter = &mut ctx.accounts.counter;
        counter.count = initial_value;
        counter.authority = ctx.accounts.authority.key();

        msg!("Counter initialized with value: {}", initial_value);
        Ok(())
    }

    /// Increment the counter by 1
    pub fn increment_counter(ctx: Context<IncrementCounter>) -> Result<()> {
        let counter = &mut ctx.accounts.counter;

        // Check for overflow
        counter.count = counter
            .count
            .checked_add(1)
            .ok_or(ErrorCode::CounterOverflow)?;

        msg!("Counter incremented to: {}", counter.count);
        Ok(())
    }

    /// Perform a CPI to increment a native counter
    /// This demonstrates how to call a native Solana program from Anchor
    pub fn increment_native_counter(ctx: Context<IncrementNativeCounter>) -> Result<()> {
        msg!("Performing CPI to native program...");

        // Native program's IncrementCounter instruction
        // Uses Borsh serialization with enum discriminator (variant 1 = IncrementCounter)
        let instruction_data: Vec<u8> = vec![1]; // Enum variant 1

        // Build the CPI instruction
        let cpi_instruction = Instruction {
            program_id: ctx.accounts.native_program.key(),
            accounts: vec![AccountMeta::new(ctx.accounts.native_counter.key(), false)],
            data: instruction_data,
        };

        // Invoke the CPI
        invoke(
            &cpi_instruction,
            &[
                ctx.accounts.native_counter.to_account_info(),
                ctx.accounts.native_program.to_account_info(),
            ],
        )?;

        msg!("Successfully incremented native counter via CPI");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeCounter<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Counter::INIT_SPACE
    )]
    pub counter: Account<'info, Counter>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct IncrementCounter<'info> {
    #[account(mut)]
    pub counter: Account<'info, Counter>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct IncrementNativeCounter<'info> {
    /// CHECK: This is the native counter account from the native program
    #[account(mut)]
    pub native_counter: AccountInfo<'info>,

    /// CHECK: This is the native program's ID
    pub native_program: AccountInfo<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Counter overflow occurred")]
    CounterOverflow,
}
