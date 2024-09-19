use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, Burn, burn, TransferChecked, transfer_checked}};

use crate::{assert_non_zero, assert_not_expired, assert_not_locked, states::Market};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    user: Signer<'info>,
    mint_yes: Box<InterfaceAccount<'info, Mint>>,
    mint_no: Box<InterfaceAccount<'info, Mint>>,
    mint_stablecoin: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"lp", market.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = auth,
    )]
    mint_lp: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = mint_yes,
        associated_token::authority = auth
    )]
    vault_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_no,
        associated_token::authority = auth
    )]
    vault_no: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_stablecoin,
        associated_token::authority = auth
    )]
    vault_stablecoin: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_stablecoin,
        associated_token::authority = auth
    )]
    user_ata_stablecoin: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_yes,
        associated_token::authority = auth
    )]
    user_ata_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_no,
        associated_token::authority = auth
    )]
    user_ata_no: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    user_ata_lp: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: this is safe
    #[account(
        seeds = [b"auth"],
        bump = market.auth_bump
    )]
    auth: UncheckedAccount<'info>,
    #[account(
        has_one = mint_yes,
        has_one = mint_no,
        has_one = mint_stablecoin,
        mut,
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
            from: self.vault_stablecoin.to_account_info(),
            mint: self.mint_stablecoin.to_account_info(),
            to: self.user_ata_stablecoin.to_account_info(),
            authority: self.auth.to_account_info()
        };

        let seeds = &[&b"auth"[..], &[self.market.auth_bump]];

        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );

        transfer_checked(ctx, amount, self.mint_stablecoin.decimals)
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
            authority: self.auth.to_account_info()
        };

        let seeds = &[&b"auth"[..], &[self.market.auth_bump]];

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