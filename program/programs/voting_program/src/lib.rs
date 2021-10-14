use anchor_lang::prelude::*;

pub mod instruction_api;

declare_id!("4dKeVRjqyVNA3n48d1RGf3k2f8fEo1fGsUMPSmsHW4LG");

#[program]
pub mod voting_program {
    use super::*;

    // ------ Instructions ------

    /// Starts the voting by creating and populating a VotingState account.
    pub fn init_voting(ctx: Context<InitVoting>) -> ProgramResult {
        msg!("init_voting started.");

        let voting_owner = &ctx.accounts.voting_owner;
        let voting_state = &mut ctx.accounts.voting_state;

        if !voting_owner.is_signer {
            Err(ProgramError::MissingRequiredSignature)?;
        }
        if voting_state.is_initialized {
            Err(ProgramError::AccountAlreadyInitialized)?
        }
        
        voting_state.is_initialized = true;
        voting_state.deadline = Clock::get()?.unix_timestamp + 7 * 86_400;
        voting_state.voting_owner = *voting_owner.key;
    
        msg!("VotingState initialized.");
    
        Ok(())
    }

    /// Makes the voter eligible for voting by creating a VoterVotes account.
    pub fn add_voter(ctx: Context<AddVoter>, voter_pubkey: Pubkey, _voter_votes_bump_seed: u8) -> ProgramResult {
        let voting_owner = &ctx.accounts.voting_owner;
        let voting_state = &ctx.accounts.voting_state;
        let voter_votes = &mut ctx.accounts.voter_votes;

        if !voting_owner.is_signer {
            Err(ProgramError::MissingRequiredSignature)?;
        }
        if voting_state.as_ref().owner != ctx.program_id {
            Err(ProgramError::IllegalOwner)?;
        }
        if voting_state.voting_owner != *voting_owner.key {
            Err(VotingError::IllegalVotingOwner)?;
        }
        if voter_votes.is_initialized {
            Err(ProgramError::AccountAlreadyInitialized)?
        }

        voter_votes.is_initialized = true;
        voter_votes.positive_votes = 2;
        voter_votes.negative_votes = 1;
        voter_votes.voter_pubkey = voter_pubkey;
        voter_votes.voting_state_pubkey = *voting_state.as_ref().key;

        msg!("VoterVotes account initialized.");

        Ok(())
    }

    /// Creates a new Party account with the requested name
    /// and increments the parties counter in the VotingState account.
    pub fn add_party(ctx: Context<AddParty>, name: String, _party_bump_seed: u8) -> ProgramResult {
        let fee_payer = &ctx.accounts.fee_payer;
        let party = &mut ctx.accounts.party;
        let voting_state = &mut ctx.accounts.voting_state;

        if !fee_payer.is_signer {
            Err(ProgramError::MissingRequiredSignature)?;
        }
        if !party.is_initialized {
            Err(ProgramError::AccountAlreadyInitialized)?;
        }
        if voting_state.deadline < Clock::get()?.unix_timestamp {
            Err(VotingError::VoteIsOver)?;
        }

        voting_state.party_count += 1;

        party.is_initialized = true;
        party.name = name;
        party.voting_state_pubkey = *voting_state.as_ref().key;

        msg!("Party account initialized.");

        Ok(())
    }

    /// Votes the provided party and creates a VoterVoted account.
    /// 
    /// The party will receive one negative or positive vote.
    pub fn vote(ctx: Context<Vote>, positive: bool, _voter_votes_bump_seed: u8) -> ProgramResult {
        let voter = &ctx.accounts.voter; 
        let voting_state = &ctx.accounts.voting_state;
        let voter_voted = &mut ctx.accounts.voter_voted;
        let voter_votes = &mut ctx.accounts.voter_votes;
        let party = &mut ctx.accounts.party;
        
        if !voter.is_signer {
            Err(ProgramError::MissingRequiredSignature)?;
        }
        if voting_state.as_ref().owner != ctx.program_id {
            Err(ProgramError::IllegalOwner)?;
        }
        if voting_state.deadline < Clock::get()?.unix_timestamp {
            Err(VotingError::VoteIsOver)?;
        }
        if voter_voted.is_initialized {
            Err(VotingError::AlreadyVoted)?;
        }
        if !voter_votes.is_initialized {
            Err(VotingError::NotEligibleForVoting)?;
        }

        if positive {
            if voter_votes.positive_votes == 0 {
                Err(VotingError::NoPositiveVotes)?;
            }
        } else {
            if voter_votes.negative_votes == 0 {
                Err(VotingError::NoNegativeVotes)?;
            }
            if voter_votes.positive_votes != 0 {
                Err(VotingError::PositiveVotesNotSpent)?;
            }
        }

        if !party.is_initialized {
            Err(ProgramError::UninitializedAccount)?;
        }

        if voter_votes.voter_pubkey != *voter.key {
            Err(VotingError::IllegalVoter)?;
        }
        if voter_votes.voting_state_pubkey != *voting_state.as_ref().key {
            Err(VotingError::IllegalVotingState)?;
        }
        if party.voting_state_pubkey != *voting_state.as_ref().key {
            Err(VotingError::IllegalVotingState)?;
        }

        if positive {
            voter_votes.positive_votes -= 1;
            party.positive_votes += 1;
        } else {
            voter_votes.negative_votes -= 1;
            party.negative_votes += 1;
        }

        voter_voted.is_initialized = true;
        voter_voted.voter_pubkey = *voter.as_ref().key;
        voter_voted.voting_state_pubkey = *voting_state.as_ref().key;

        msg!("Voted.");

        Ok(())
    }
}

// ------ Instruction Accounts ------

// @TODO relations + size?, etc.

#[derive(Accounts)]
pub struct InitVoting<'info> {
    #[account(mut)]
    pub voting_owner: Signer<'info>,
    #[account(mut)]
    pub voting_state: Account<'info, VotingState>,
}

#[derive(Accounts)]
#[instruction(voter_pubkey: Pubkey, _voter_votes_bump_seed: u8)]
pub struct AddVoter<'info> {
    #[account(mut)]
    pub voting_owner: Signer<'info>,
    pub voting_state: Account<'info, VotingState>,
    #[account(init, payer = voting_owner, seeds = [
        b"voter_votes",
        voter_pubkey.as_ref(),
        voting_state.as_ref().key.as_ref(),
    ], bump = _voter_votes_bump_seed)]
    pub voter_votes: Account<'info, VoterVotes>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(name: String, _party_bump_seed: u8)]
pub struct AddParty<'info> {
    #[account(mut)]
    pub fee_payer: Signer<'info>,
    #[account(init, payer = fee_payer, seeds = [
        b"party",
        voting_state.party_count.to_le_bytes().as_ref(),
        voting_state.as_ref().key.as_ref(),
    ], bump = _party_bump_seed)]
    pub party: Account<'info, Party>,
    #[account(mut)]
    pub voting_state: Account<'info, VotingState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(positive: bool, _voter_votes_bump_seed: u8)]
pub struct Vote<'info> {
    pub voter: Signer<'info>,
    pub voting_state: Account<'info, VotingState>,
    #[account(init, payer = voter, seeds = [
        b"voter_voted",
        voter.as_ref().key.as_ref(),
        party.as_ref().key.as_ref(),
        voting_state.as_ref().key.as_ref(),
    ], bump = _voter_votes_bump_seed)]
    pub voter_voted: Account<'info, VoterVoted>,
    #[account(mut)]
    pub voter_votes: Account<'info, VoterVotes>,
    #[account(mut)]
    pub party: Account<'info, Party>,
    pub system_program: Program<'info, System>,
}

// ------ Account States (Data) ------

#[account]
#[derive(Default, Debug)]
pub struct VotingState {
    pub is_initialized: bool,
    pub deadline: i64,
    pub party_count: u32,
    pub voting_owner: Pubkey,
}
// @TODO remove (?)
impl VotingState {
    pub fn serialized_size() -> usize {
        // @TODO compute once?
        8 + Self::default()
            .try_to_vec()
            .expect("failed to serialize default VotingState")
            .len()
    }
}

#[account]
#[derive(Default, Debug)]
pub struct VoterVotes {
    pub is_initialized: bool,
    pub positive_votes: u8,
    pub negative_votes: u8,
    pub voter_pubkey: Pubkey,
    pub voting_state_pubkey: Pubkey,
}

#[account]
#[derive(Default, Debug)]
pub struct VoterVoted {
    pub is_initialized: bool,
    pub voter_pubkey: Pubkey,
    pub voting_state_pubkey: Pubkey,
}

#[account]
#[derive(Default, Debug)]
pub struct Party {
    pub is_initialized: bool,
    pub positive_votes: u32,
    pub negative_votes: u32,
    pub name: String,
    pub voting_state_pubkey: Pubkey,
}

// ------ Errors ------

#[error]
pub enum VotingError {
    #[msg("invalid Instruction")]
    InvalidInstruction,

    #[msg("illegal voting owner")]
    IllegalVotingOwner,

    #[msg("illegal voter")]
    IllegalVoter,

    #[msg("illegal voting state")]
    IllegalVotingState,

    #[msg("the vote is over")]
    VoteIsOver,

    #[msg("the voter is not eligible for voting")]
    NotEligibleForVoting,

    #[msg("the voter already voted for the selected party")]
    AlreadyVoted,

    #[msg("no positive votes left")]
    NoPositiveVotes,

    #[msg("no negative votes left")]
    NoNegativeVotes,

    #[msg("positive votes have to be spent before the negative ones")]
    PositiveVotesNotSpent,
}
