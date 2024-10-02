use anchor_lang::prelude::*;

use crate::states::Market;

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
        self.market.locked = true;
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<()> {
        self.market.locked = false;
        Ok(())
    }
}