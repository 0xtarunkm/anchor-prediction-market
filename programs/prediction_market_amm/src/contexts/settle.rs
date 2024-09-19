use anchor_lang::prelude::*;

use crate::{assert_not_locked, error::MarketError, states::Market};

#[derive(Accounts)]
pub struct SettleMarket<'info> {
    #[account(mut)]
    admin: Signer<'info>,
    #[account(
        mut,
        seeds = [b"market", market.seed.to_le_bytes().as_ref()],
        bump = market.market_bump,
    )]
    pub market: Account<'info, Market>,
}

impl<'info> SettleMarket<'info> {
    pub fn settle(&mut self, is_resolved: bool) -> Result<()> {
        assert_not_locked!(self.market.locked);

        require!(self.market.settled == true, MarketError::MarketAlreadySettled);

        require!(Clock::get()?.unix_timestamp > self.market.end_time, MarketError::MarketNotEnded);

        if is_resolved {
            self.market.settled = true;
        } else {
            self.market.settled = false;
        }
        Ok(())
    }
}