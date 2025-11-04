use anchor_lang::prelude::*;

declare_id!("27YreJqker2o5TvzzLUsiC9ZGMdPThvEm8qZBNDw5EWX");

#[program]
pub mod anchor_counter {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
