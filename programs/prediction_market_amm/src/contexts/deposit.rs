use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked, MintTo, mint_to}};
use prediction_market_curve::{OutcomeToken, PredictionMarket};

use crate::{assert_non_zero, assert_not_expired, assert_not_locked, error::MarketError, states:: Market};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    user: Signer<'info>,
    mint_yes: Box<InterfaceAccount<'info, Mint>>,
    mint_no: Box<InterfaceAccount<'info, Mint>>,
    mint_stablecoin: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init_if_needed,
        seeds = [b"lp", market.key().as_ref()],
        bump,
        payer = user,
        mint::decimals = 6,
        mint::authority = auth
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

impl<'info> Deposit<'info> {
    pub fn deposit(
        &mut self,
        amount: u64,
        max_yes: u64,
        max_no: u64,
        expiration: i64,
    ) -> Result<()> {
        assert_not_locked!(self.market.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([amount, max_yes, max_no]);

        let (yes_amount, no_amount, lp_tokens, _) = match self.mint_lp.supply == 0 && self.vault_no.amount == 0 && self.vault_yes.amount == 0 {
            true =>  (max_yes, max_no, amount, amount),
            false => {
                let market = PredictionMarket::add_liquidity(&mut self, amount).unwrap();

                (market.yes_amount, market.no_amount, market.lp_tokens, market.usdc_amount)
            }
        };

        require!(yes_amount<=max_yes && no_amount<=max_no, MarketError::SlippageExceeded);

        self.deposit_tokens(yes_amount, OutcomeToken::YES)?;
        self.deposit_tokens(no_amount, OutcomeToken::NO)?;
        self.mint_lp_tokens(lp_tokens)
    }

    pub fn deposit_tokens(
        &mut self,
        amount: u64,
        token: OutcomeToken,
    ) -> Result<()> {
        let mint;
        let (from, to) = match token {
            OutcomeToken::YES => {
                mint = self.mint_yes.clone();
                (self.user_yes.to_account_info(), self.vault_yes.to_account_info())
            },
            OutcomeToken::NO => {
                mint = self.mint_no.clone();
                (self.user_no.to_account_info(), self.vault_no.to_account_info())
            },
        };
        
        let cpi_account = TransferChecked {
            from,
            mint: mint.to_account_info(),
            to,
            authority: self.user.to_account_info()
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_account);
        
        transfer_checked(ctx, amount, 6)
    }

    pub fn mint_lp_tokens(
        &self,
        amount: u64,
    ) -> Result<()> {
        let accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_lp.to_account_info(),
            authority: self.auth.to_account_info(),
        };
        let seeds = &[
            &b"auth"[..],
            &[self.market.auth_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, signer_seeds);
        mint_to(ctx, amount)
    }
}