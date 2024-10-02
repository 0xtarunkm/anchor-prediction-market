use anchor_lang::prelude::*;

mod contexts;
mod error;
mod helpers;
mod states;

use contexts::*;

declare_id!("8vxoTyexgHLcNjo7kcRuF22uWp8fDKtK3yfehJY4Lont");

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
        ctx.accounts.save_market(seed, name, fee, end_time, &ctx.bumps)
    }

    // add liquidity to mint LP tokens
    pub fn add_liquidity(
        ctx: Context<Deposit>,
        amount: u64,
        max_yes: u64,
        max_no: u64,
        expiration: i64,
    ) -> Result<()> {
        ctx.accounts.deposit(amount, max_no, max_yes, expiration)
    }

    // burn lp tokens to withdraw liquidity
    pub fn withdraw_liquidity(ctx: Context<Withdraw>, amount: u64, expiration: i64) -> Result<()> {
        ctx.accounts.withdraw(amount, expiration)
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
