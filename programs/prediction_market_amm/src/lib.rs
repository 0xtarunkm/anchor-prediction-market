use anchor_lang::prelude::*;

mod states;
mod contexts;
mod helpers;
mod error;

use contexts::*;

declare_id!("HEqjCVX5AHi9kYFF955HbEeTF95DUdz1aZTviRucL16d");

#[program]
pub mod prediction_market_amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, seed: u64, name: String, fee: u16, authority: Option<Pubkey>) -> Result<()> {
        ctx.accounts.init(seed, name, fee, authority, &ctx.bumps)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, max_yes: u64, max_no: u64, expiration: i64) -> Result<()> {
        ctx.accounts.deposit(amount, max_yes, max_no, expiration)
    }
}