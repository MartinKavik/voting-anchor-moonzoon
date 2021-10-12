use anchor_lang::prelude::*;

declare_id!("4dKeVRjqyVNA3n48d1RGf3k2f8fEo1fGsUMPSmsHW4LG");

#[program]
pub mod voting_program {
    use super::*;
    pub fn initialize(_ctx: Context<Initialize>) -> ProgramResult {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
