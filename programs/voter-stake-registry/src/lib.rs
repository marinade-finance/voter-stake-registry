use account::*;
use anchor_lang::prelude::*;
use error::*;
use instructions::*;

#[macro_use]
pub mod account;
mod error;
mod instructions;

// The program address.
declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// # Introduction
///
/// The governance registry is an "addin" to the SPL governance program that
/// allows one to both vote with many different ypes of tokens for voting and to
/// scale voting power as a linear function of time locked--subject to some
/// maximum upper bound.
///
/// The flow for voting with this program is as follows:
///
/// - Create a SPL governance realm.
/// - Create a governance registry account.
/// - Add exchange rates for any tokens one wants to deposit. For example,
///   if one wants to vote with tokens A and B, where token B has twice the
///   voting power of token A, then the exchange rate of B would be 2 and the
///   exchange rate of A would be 1.
/// - Create a voter account.
/// - Deposit tokens into this program, with an optional lockup period.
/// - Vote.
///
/// Upon voting with SPL governance, a client is expected to call
/// `decay_voting_power` to get an up to date measurement of a given `Voter`'s
/// voting power for the given slot. If this is not done, then the transaction
/// will fail (since the SPL governance program will require the measurement
/// to be active for the current slot).
///
/// # Interacting with SPL Governance
///
/// This program does not directly interact with SPL governance via CPI.
/// Instead, it simply writes a `VoterWeightRecord` account with a well defined
/// format, which is then used by SPL governance as the voting power measurement
/// for a given user.
///
/// # Max Vote Weight
///
/// Given that one can use multiple tokens to vote, the max vote weight needs
/// to be a function of the total supply of all tokens, converted into a common
/// currency. For example, if you have Token A and Token B, where 1 Token B =
/// 10 Token A, then the `max_vote_weight` should be `supply(A) + supply(B)*10`
/// where both are converted into common decimals. Then, when calculating the
/// weight of an individual voter, one can convert B into A via the given
/// exchange rate, which must be fixed.
///
/// Note that the above also implies that the `max_vote_weight` must fit into
/// a u64.
#[program]
pub mod voter_stake_registry {
    use super::*;

    pub fn create_registrar(
        ctx: Context<CreateRegistrar>,
        vote_weight_decimals: u8,
        registrar_bump: u8,
    ) -> Result<()> {
        instructions::create_registrar(ctx, vote_weight_decimals, registrar_bump)
    }

    pub fn create_exchange_rate(
        ctx: Context<CreateExchangeRate>,
        idx: u16,
        rate: u64,
        decimals: u8,
    ) -> Result<()> {
        instructions::create_exchange_rate(ctx, idx, rate, decimals)
    }

    pub fn create_voter(
        ctx: Context<CreateVoter>,
        voter_bump: u8,
        voter_weight_record_bump: u8,
    ) -> Result<()> {
        instructions::create_voter(ctx, voter_bump, voter_weight_record_bump)
    }

    pub fn create_deposit_entry(
        ctx: Context<CreateDepositEntry>,
        kind: LockupKind,
        periods: i32,
        allow_clawback: bool,
    ) -> Result<()> {
        instructions::create_deposit_entry(ctx, kind, periods, allow_clawback)
    }

    pub fn deposit(ctx: Context<Deposit>, id: u8, amount: u64) -> Result<()> {
        instructions::deposit(ctx, id, amount)
    }

    pub fn withdraw(ctx: Context<WithdrawOrClawback>, deposit_id: u8, amount: u64) -> Result<()> {
        instructions::withdraw(ctx, deposit_id, amount)
    }

    pub fn clawback(ctx: Context<WithdrawOrClawback>, deposit_id: u8) -> Result<()> {
        instructions::clawback(ctx, deposit_id)
    }

    pub fn close_deposit_entry(ctx: Context<CloseDepositEntry>, deposit_id: u8) -> Result<()> {
        instructions::close_deposit_entry(ctx, deposit_id)
    }

    pub fn reset_lockup(ctx: Context<ResetLockup>, deposit_id: u8, periods: i64) -> Result<()> {
        instructions::reset_lockup(ctx, deposit_id, periods)
    }

    pub fn update_voter_weight_record(ctx: Context<UpdateVoterWeightRecord>) -> Result<()> {
        instructions::update_voter_weight_record(ctx)
    }

    pub fn update_max_vote_weight<'info>(ctx: Context<UpdateMaxVoteWeight>) -> Result<()> {
        instructions::update_max_vote_weight(ctx)
    }

    pub fn close_voter(ctx: Context<CloseVoter>) -> Result<()> {
        instructions::close_voter(ctx)
    }

    pub fn set_time_offset(ctx: Context<SetTimeOffset>, time_offset: i64) -> Result<()> {
        instructions::set_time_offset(ctx, time_offset)
    }
}