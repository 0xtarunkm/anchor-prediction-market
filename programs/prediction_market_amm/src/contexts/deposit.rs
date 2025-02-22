use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{mint_to, Mint, MintTo, TokenAccount, TokenInterface},
};

use crate::{assert_non_zero, assert_not_expired, assert_not_locked, states::Market};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    user: Signer<'info>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = market
    )]
    mint_yes: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = market
    )]
    mint_no: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        mint::token_program = token_program,
    )]
    mint_usdc: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = mint_yes,
        associated_token::authority = market,
        associated_token::token_program = token_program
    )]
    vault_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_no,
        associated_token::authority = market,
        associated_token::token_program = token_program
    )]
    vault_no: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = market,
        associated_token::token_program = token_program
    )]
    vault_usdc: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        has_one = mint_yes,
        has_one = mint_no,
        seeds = [b"market", market.seed.to_le_bytes().as_ref()],
        bump = market.market_bump
    )]
    market: Box<Account<'info, Market>>,

    token_program: Interface<'info, TokenInterface>,
    associated_token_program: Program<'info, AssociatedToken>,
    system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, min_no: u64, min_yes: u64, expiration: i64) -> Result<()> {
        assert_not_locked!(self.market.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([min_yes, min_no]);

        self.mint_token(min_yes, true)?;
        self.mint_token(min_no, false)?;

        self.market
            .total_liquidity
            .checked_add(min_yes.checked_add(min_no).unwrap())
            .unwrap();

        Ok(())
    }

    pub fn mint_token(&mut self, amount: u64, is_yes: bool) -> Result<()> {
        let (to, mint) = match is_yes {
            true => (
                self.vault_yes.to_account_info(),
                self.mint_yes.to_account_info(),
            ),
            false => (
                self.vault_no.to_account_info(),
                self.mint_no.to_account_info(),
            ),
        };

        let cpi_account = MintTo {
            mint,
            to,
            authority: self.market.to_account_info(),
        };

        let seeds = &[
            &b"market"[..],
            &self.market.seed.to_le_bytes(),
            &[self.market.market_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_account,
            signer_seeds,
        );

        mint_to(ctx, amount)
    }
}
