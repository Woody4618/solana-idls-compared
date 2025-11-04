use anchor_lang::prelude::*;

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
