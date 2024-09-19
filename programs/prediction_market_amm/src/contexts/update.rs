use anchor_lang::prelude::*;

use crate::{has_update_authority, states::Market};

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"market", market.seed.to_le_bytes().as_ref()],
        bump = market.market_bump,
    )]
    market: Box<Account<'info, Market>>
}

impl<'info> Update<'info> {
    pub fn lock(&mut self) -> Result<()> {
        has_update_authority!(self);
        self.market.locked = true;
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<()> {
        has_update_authority!(self);
        self.market.locked = false;
        Ok(())
    }
}