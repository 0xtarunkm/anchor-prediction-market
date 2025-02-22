use anchor_lang::prelude::*;

mod contexts;
mod error;
mod helpers;
mod states;

use contexts::*;

declare_id!("5KXREawEvTiDAgfWH2UWVERCani5WnjJx4ZQuXardgEW");

#[program]
pub mod prediction_market_amm {

    use super::*;

    #[inline(never)]
    pub fn initialize(
        ctx: Context<Initialize>,
        seed: u64,
        name: String,
        fee: u16,
        end_time: i64,
    ) -> Result<()> {
        ctx.accounts
            .save_market(seed, name, fee, end_time, &ctx.bumps)
    }

    pub fn add_liquidity(
        ctx: Context<Deposit>,
        max_yes: u64,
        max_no: u64,
        expiration: i64,
    ) -> Result<()> {
        ctx.accounts.deposit(max_no, max_yes, expiration)
    }

    pub fn swap(
        ctx: Context<Swap>,
        is_usdc_to_token: bool,
        amount: u64,
        is_yes: bool,
        min_out: u64,
        expiration: i64,
    ) -> Result<()> {
        ctx.accounts
            .swap(is_usdc_to_token, amount, is_yes, min_out, expiration)
    }

    pub fn settle(ctx: Context<SettleMarket>, is_resolved: bool) -> Result<()> {
        ctx.accounts.settle(is_resolved)
    }

    pub fn claim(ctx: Context<ClaimReward>, is_yes: bool) -> Result<()> {
        ctx.accounts.claim(is_yes)
    }

    pub fn lock(ctx: Context<Update>) -> Result<()> {
        ctx.accounts.lock()
    }

    pub fn unlock(ctx: Context<Update>) -> Result<()> {
        ctx.accounts.unlock()
    }
}
