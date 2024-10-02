use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, Burn, burn, TransferChecked, transfer_checked}};

use crate::{assert_non_zero, assert_not_expired, assert_not_locked, states::Market};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    user: Signer<'info>,
    mint_yes: Box<InterfaceAccount<'info, Mint>>,
    mint_no: Box<InterfaceAccount<'info, Mint>>,
    mint_usdc: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"lp", market.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = market,
        mint::token_program = token_program
    )]
    mint_lp: Box<InterfaceAccount<'info, Mint>>,
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
        associated_token::authority = market
    )]
    vault_usdc: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = user
    )]
    user_ata_usdc: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_yes,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    user_ata_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_no,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    user_ata_no: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    user_ata_lp: Box<InterfaceAccount<'info, TokenAccount>>,
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
    system_program: Program<'info, System>
}

impl<'info> Withdraw<'info> {
    pub fn withdraw(&self, lp_amount: u64, expiration: i64) -> Result<()> {
        assert_not_locked!(self.market.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([lp_amount]);

        let liquidity_fraction = lp_amount as f64 / self.mint_lp.supply as f64;

        let usdc_to_return = (self.market.total_liquidity as f64 * liquidity_fraction) as u64;
        let tokens_to_return = usdc_to_return / 2;

        self.burn_lp_tokens(lp_amount)?;
        self.withdraw_tokens(true, tokens_to_return)?;
        self.withdraw_tokens(false, tokens_to_return)?;

        self.market.total_liquidity.checked_sub(usdc_to_return).unwrap();

        self.withdraw_stablecoin(usdc_to_return)
    }

    pub fn withdraw_stablecoin(&self, amount: u64) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.vault_usdc.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.user_ata_usdc.to_account_info(),
            authority: self.market.to_account_info()
        };

        let seeds = &[
            &b"market"[..],
            &self.market.seed.to_le_bytes(),
            &[self.market.market_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        transfer_checked(ctx, amount, self.mint_usdc.decimals)
    }

    pub fn withdraw_tokens(&self, is_yes: bool, amount: u64) -> Result<()> {

        let (from, to, mint, decimals) = match is_yes {
            true => ( self.vault_yes.to_account_info(), self.user_ata_yes.to_account_info(), self.mint_yes.to_account_info(), self.mint_yes.decimals),
            false => (self.vault_no.to_account_info(), self.user_ata_no.to_account_info(), self.mint_no.to_account_info(), self.mint_yes.decimals)
        };

        let cpi_accounts = TransferChecked {
            from,
            mint,
            to,
            authority: self.market.to_account_info()
        };

        let seeds = &[
            &b"market"[..],
            &self.market.seed.to_le_bytes(),
            &[self.market.market_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        transfer_checked(ctx, amount, decimals)
    }

    pub fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_ata_lp.to_account_info(),
            authority: self.user.to_account_info()
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        burn(ctx, amount)
    }
}