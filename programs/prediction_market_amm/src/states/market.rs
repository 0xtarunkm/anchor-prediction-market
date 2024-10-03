use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Market {
    #[max_len(32)]
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