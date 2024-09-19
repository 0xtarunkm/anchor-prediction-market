use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Market {
    #[max_len(100)]
    pub market_name: String,
    pub authority: Option<Pubkey>,
    pub seed: u64,
    pub mint_yes: Pubkey,
    pub mint_no: Pubkey,
    pub mint_stablecoin: Pubkey,
    pub total_liquidity: u64,
    pub fee: u16,
    pub locked: bool,
    pub settled: bool,
    pub auth_bump: u8,
    pub market_bump: u8
}