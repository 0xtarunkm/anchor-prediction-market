use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{assert_non_zero, assert_not_expired, assert_not_locked, error::MarketError, helpers::calculate_output, states::Market};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    user: Signer<'info>,    
    #[account(
        mut,
        seeds = [b"yes", market.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = auth,
    )]
    mint_yes: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        seeds = [b"no", market.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = auth,
    )]
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
        associated_token::authority = auth,
    )]
    vault_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_no,
        associated_token::authority = auth,
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
        associated_token::mint = mint_yes,
        associated_token::authority = user,
    )]
    user_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_no,
        associated_token::authority = user,
    )]
    user_no: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_stablecoin,
        associated_token::authority = user,
    )]
    user_stablecoin: Box<InterfaceAccount<'info, TokenAccount>>,
    /// CHECK: this is safe
    #[account(
        seeds = [b"auth"],
        bump = market.auth_bump
    )]
    auth: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"market", market.seed.to_le_bytes().as_ref()],
        bump = market.market_bump,
    )]
    pub market: Box<Account<'info, Market>>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn swap(
        &mut self,
        is_usdc_to_token: bool,
        amount_in: u64,
        is_yes: bool,
        min_out: u64,
        expiration: i64
    ) -> Result<()> {
        assert_not_locked!(self.market.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([amount_in, min_out]);

        let amount_out = if is_usdc_to_token {
            if is_yes {
                calculate_output(amount_in, self.vault_stablecoin.amount, self.vault_yes.amount)
            } else {
                calculate_output(amount_in, self.vault_stablecoin.amount, self.vault_no.amount)
            }
        } else {
            if is_yes {
                calculate_output(amount_in, self.vault_yes.amount, self.vault_stablecoin.amount)
            } else {
                calculate_output(amount_in, self.vault_no.amount, self.vault_stablecoin.amount)
            }
        };

        require!(amount_out < min_out, MarketError::SlippageExceeded);

        if is_usdc_to_token {
            self.deposit_tokens(true, None, amount_in)?;
            self.withdraw_token(false, amount_out, Some(is_yes))
        } else {
            self.deposit_tokens(false, Some(is_yes), amount_in)?;
            self.withdraw_token(true, amount_out, None)
        }
    }

    pub fn deposit_tokens(
        &mut self,
        is_usdc: bool,
        is_yes: Option<bool>,
        amount: u64
    ) -> Result<()> {
        let (mint, from, to, decimals) = match is_usdc {
            true => (
                self.mint_stablecoin.to_account_info(),
                self.user_stablecoin.to_account_info(),
                self.vault_stablecoin.to_account_info(),
                self.mint_stablecoin.decimals
            ),
            false => {
                match is_yes {
                    Some(true) => (
                        self.mint_yes.to_account_info(),
                        self.user_yes.to_account_info(),
                        self.vault_yes.to_account_info(),
                        self.mint_yes.decimals
                    ),
                    Some(false) => (
                        self.mint_no.to_account_info(),
                        self.user_no.to_account_info(),
                        self.vault_no.to_account_info(),
                        self.mint_no.decimals
                    ),
                    None => return Err(MarketError::InvalidToken.into())
                }
            }
        };

        let account = TransferChecked {
            from,
            mint,
            to,
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), account);

        transfer_checked(ctx, amount, decimals)
    }

    pub fn withdraw_token(
        &mut self,
        is_usdc: bool,
        amount: u64,
        is_yes: Option<bool>
    ) -> Result<()> {
        let (mint, from, to, decimals) = match is_usdc {
            true => (
                self.mint_stablecoin.to_account_info(),
                self.vault_stablecoin.to_account_info(),
                self.user_stablecoin.to_account_info(),
                self.mint_stablecoin.decimals
            ),
            false => {
                match is_yes {
                    Some(true) => (
                        self.mint_yes.to_account_info(),
                        self.vault_yes.to_account_info(),
                        self.user_yes.to_account_info(),
                        self.mint_yes.decimals
                    ),
                    Some(false) => (
                        self.mint_no.to_account_info(),
                        self.vault_no.to_account_info(),
                        self.user_no.to_account_info(),
                        self.mint_no.decimals
                    ),
                    None => return Err(MarketError::InvalidToken.into())
                }
            }
        };

        let account = TransferChecked {
            from,
            mint,
            to,
            authority: self.auth.to_account_info()
        };

        let seeds = &[
            &b"auth"[..],
            &[self.market.auth_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), account, signer_seeds);

        transfer_checked(ctx, amount, decimals)
    }
}