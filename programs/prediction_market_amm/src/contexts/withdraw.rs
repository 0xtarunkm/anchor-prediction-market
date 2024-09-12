use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, Burn, burn, TransferChecked, transfer_checked}};
use prediction_market_curve::{OutcomeToken, PredictionMarket};

use crate::{error::MarketError, states::Market};

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
    user_stablecoin: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_yes,
        associated_token::authority = auth
    )]
    user_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_no,
        associated_token::authority = auth
    )]
    user_no: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_lp,
        associated_token::authority = user,
    )]
    user_lp: Box<InterfaceAccount<'info, TokenAccount>>,
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
    pub fn withdraw(&self, amount: u64, min_yes: u64, min_no: u64, _expiration: i64) -> Result<()> {
        let amounts = PredictionMarket::remove_liquidity(amount, 10_000, self.vault_yes.amount, self.vault_no.amount, self.vault_stablecoin.amount, self.mint_lp.supply).map_err(MarketError::from)?;

        require!(
            min_yes <= amounts.yes_amount && min_no <= amounts.no_amount,
            MarketError::SlippageExceeded
        );

        self.burn_lp_tokens(amount)?;
        self.withdraw_tokens(OutcomeToken::YES, amounts.yes_amount)?;
        self.withdraw_tokens(OutcomeToken::NO, amounts.no_amount)?;
        self.withdraw_stablecoin(amounts.usdc_amount)
    }

    pub fn withdraw_stablecoin(&self, amount: u64) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.vault_stablecoin.to_account_info(),
            mint: self.mint_stablecoin.to_account_info(),
            to: self.user_stablecoin.to_account_info(),
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

    pub fn withdraw_tokens(&self, token: OutcomeToken, amount: u64) -> Result<()> {
        let mint;
        let (from, to) = match token {
            OutcomeToken::YES => {
                mint = self.mint_yes.clone();
                (
                self.vault_yes.to_account_info(),
                self.user_yes.to_account_info()
                )
            },
            OutcomeToken::NO => {
                mint = self.mint_no.clone();
                (
                    self.vault_no.to_account_info(),
                    self.user_no.to_account_info()
                )
            }
        };

        let cpi_accounts = TransferChecked {
            from,
            mint: mint.to_account_info(),
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

        transfer_checked(ctx, amount, mint.decimals)
    }

    pub fn burn_lp_tokens(&self, amount: u64) -> Result<()> {
        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.user.to_account_info()
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        burn(ctx, amount)
    }
}