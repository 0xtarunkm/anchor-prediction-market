use anchor_lang::prelude::*;

#[account]
pub struct Market {
    pub market_name: String,
    pub seed: u64,
    pub mint_yes: Pubkey,
    pub mint_no: Pubkey,
    pub total_liquidity: u64,
    pub fee: u16,
    pub locked: bool,
    pub end_time: i64,
    pub settled: bool,
    pub market_bump: u8
}

impl Space for Market {
    const INIT_SPACE: usize = 8 + (4 + 32) + 8 + (4 + 32) + (4 + 32) + 8 + 2 + 1 + 8 + 1 + 1;
}