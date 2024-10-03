use anchor_lang::prelude::*;
use anchor_spl::{associated_token::{create_idempotent, AssociatedToken}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::states::Market;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    mint_yes: Box<InterfaceAccount<'info, Mint>>,
    mint_no: Box<InterfaceAccount<'info, Mint>>,
    mint_usdc: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_yes,
        associated_token::authority = market,
    )]
    vault_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init,
        payer = signer,
        associated_token::mint = mint_no,
        associated_token::authority = market,
    )]
    vault_no: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: This account is not read or written in this instruction
    #[account(mut)]
    vault_usdc: UncheckedAccount<'info>,
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
        bumps: &InitializeBumps
    ) -> Result<()> {
        self.market.set_inner(Market { 
            market_name: name, 
            seed, 
            mint_yes: self.mint_yes.key(), 
            mint_no: self.mint_no.key(), 
            total_liquidity: 0,
            end_time,
            fee, 
            locked: false, 
            settled: false, 
            market_bump: bumps.market 
        });

        create_idempotent(
            CpiContext::new(
                self.associated_token_program.to_account_info(),
                anchor_spl::associated_token::Create {
                    payer: self.signer.to_account_info(),
                    associated_token: self.vault_usdc.to_account_info(),
                    authority: self.market.to_account_info(),
                    mint: self.mint_usdc.to_account_info(),
                    system_program: self.system_program.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                }
            )
        )?;

        Ok(())
    }
}