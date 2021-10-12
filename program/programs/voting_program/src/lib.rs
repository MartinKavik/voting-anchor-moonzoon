use anchor_lang::prelude::*;

declare_id!("4dKeVRjqyVNA3n48d1RGf3k2f8fEo1fGsUMPSmsHW4LG");

#[program]
pub mod voting_program {
    use super::*;

    // ------ Instructions ------

    /// Starts the voting by creating and populating a VotingState account.
    pub fn init_voting(_ctx: Context<InitVoting>) -> ProgramResult {
        Ok(())
    }

    /// Makes the voter eligible for voting by creating a VoterVotes account.
    pub fn add_voter(_ctx: Context<AddVoter>, voter_pubkey: Pubkey, voter_votes_bump_seed: u8) -> ProgramResult {
        Ok(())
    }

    /// Creates a new Party account with the requested name
    /// and increments the parties counter in the VotingState account.
    pub fn add_party(_ctx: Context<AddParty>, name: String, party_bump_seed: u8) -> ProgramResult {
        Ok(())
    }

    /// Votes the provided party and creates a VoterVoted account.
    /// 
    /// The party will receive one negative or positive vote.
    pub fn vote(_ctx: Context<Vote>, positive: bool, voter_votes_bump_seed: u8) -> ProgramResult {
        Ok(())
    }
}

// ------ Instruction Accounts ------

// @TODO relations + seeds + init, etc.

#[derive(Accounts)]
pub struct InitVoting<'info> {
    #[account(mut)]
    pub voting_owner: Signer<'info>,
    #[account(mut)]
    pub voting_state: Account<'info, VotingState>,
}

#[derive(Accounts)]
pub struct AddVoter<'info> {
    #[account(mut)]
    pub voting_owner: Signer<'info>,
    pub voting_state: Account<'info, VotingState>,
    // #[account(init, payer = voting_owner, space = 8 + 8)]
    #[account(mut)]
    pub voter_votes: Account<'info, VoterVotes>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddParty<'info> {
    #[account(mut)]
    pub fee_payer: Signer<'info>,
    // #[account(init, payer = fee_payer, space = 8 + 8)]
    #[account(mut)]
    pub party: Account<'info, Party>,
    #[account(mut)]
    pub voting_state: Account<'info, VotingState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Vote<'info> {
    pub voter: Signer<'info>,
    pub voting_state: Account<'info, VotingState>,
    // #[account(init, payer = voter, space = 8 + 8)]
    #[account(mut)]
    pub voter_voted: Account<'info, VoterVoted>,
    #[account(mut)]
    pub voter_votes: Account<'info, VoterVotes>,
    #[account(mut)]
    pub party: Account<'info, Party>,
    pub system_program: Program<'info, System>,
}

// ------ Account States (Data) ------

// @TODO remove is_initialized?

#[account]
#[derive(Default)]
pub struct VotingState {
    pub is_initialized: bool,
    pub deadline: i64,
    pub party_count: u32,
    pub voting_owner: Pubkey,
}

#[account]
#[derive(Default)]
pub struct VoterVotes {
    pub is_initialized: bool,
    pub positive_votes: u8,
    pub negative_votes: u8,
    pub voter_pubkey: Pubkey,
    pub voting_state_pubkey: Pubkey,
}

#[account]
#[derive(Default)]
pub struct VoterVoted {
    pub is_initialized: bool,
    pub voter_pubkey: Pubkey,
    pub voting_state_pubkey: Pubkey,
}

#[account]
#[derive(Default)]
pub struct Party {
    pub is_initialized: bool,
    pub positive_votes: u32,
    pub negative_votes: u32,
    pub name: String,
    pub voting_state_pubkey: Pubkey,
}
