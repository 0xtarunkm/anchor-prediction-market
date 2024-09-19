use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::states::Market;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    mint_yes: Box<InterfaceAccount<'info, Mint>>,
    mint_no: Box<InterfaceAccount<'info, Mint>>,
    mint_stablecoin: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_yes,
        associated_token::authority = auth
    )]
    vault_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_no,
        associated_token::authority = auth
    )]
    vault_no: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_stablecoin,
        associated_token::authority = auth
    )]
    vault_stablecoin: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: this is safe
    #[account(
        seeds = [b"auth"],
        bump
    )]
    auth: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [b"market", seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + Market::INIT_SPACE
    )]
    market: Box<Account<'info, Market>>,
    system_program: Program<'info, System>,
    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>
}

impl<'info> Initialize<'info> {
    pub fn save_market(
        &mut self,
        seed: u64,
        name: String,
        fee: u16,
        end_time: i64,
        authority: Option<Pubkey>,
        bumps: &InitializeBumps
    ) -> Result<()> {
        self.market.set_inner(Market { 
            market_name: name, 
            seed, 
            authority,
            mint_yes: self.mint_yes.key(), 
            mint_no: self.mint_no.key(), 
            mint_stablecoin: self.mint_stablecoin.key(), 
            total_liquidity: 0,
            end_time,
            fee, 
            locked: false, 
            settled: false, 
            auth_bump: bumps.auth, 
            market_bump: bumps.market 
        });

        Ok(())
    }
}