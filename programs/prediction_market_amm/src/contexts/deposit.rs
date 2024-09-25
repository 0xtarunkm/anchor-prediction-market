use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked, MintTo, mint_to}};

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

impl<'info> Deposit<'info> {
    pub fn deposit(
        &mut self,
        usdc_amount: u64,
        max_no: u64,
        max_yes: u64,
        expiration: i64,
    ) -> Result<()> {
        assert_not_locked!(self.market.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([usdc_amount, max_yes, max_no]);

        let cpi_account = TransferChecked {
            from: self.user_ata_stablecoin.to_account_info(),
            mint: self.mint_stablecoin.to_account_info(),
            to: self.vault_stablecoin.to_account_info(), 
            authority: self.user.to_account_info()
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_account);
        
        transfer_checked(ctx, usdc_amount, self.mint_stablecoin.decimals)?;

        let yes_amount = (usdc_amount).checked_div(2).unwrap();
        let no_amount = (usdc_amount).checked_div(2).unwrap();
        
        require!(yes_amount<=max_yes && no_amount<=max_no, MarketError::SlippageExceeded);

        self.mint_token(yes_amount, true)?;
        self.mint_token(no_amount, false)?;

        let lp_token_to_mint = if self.market.total_liquidity == 0 {
            usdc_amount
        } else {
            usdc_amount * self.mint_lp.supply / self.market.total_liquidity
        };

        self.market.total_liquidity.checked_add(usdc_amount).unwrap();

        self.mint_lp_tokens(lp_token_to_mint)
    }

    pub fn mint_token(
        &mut self,
        amount: u64,
        is_yes: bool,
    ) -> Result<()> {
        let (authority, to, mint) = match is_yes {
            true => (self.auth.to_account_info(), self.vault_yes.to_account_info(), self.mint_yes.to_account_info()),
            false => (self.auth.to_account_info(), self.vault_no.to_account_info(), self.mint_no.to_account_info())
        };
        
        let cpi_account = MintTo {
            mint,
            to, 
            authority
        };

        let seeds = &[
            &b"auth"[..],
            &[self.market.auth_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_account, signer_seeds);
        
        mint_to(ctx, amount)
    }

    pub fn mint_lp_tokens(
        &self,
        amount: u64,
    ) -> Result<()> {
        let accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_ata_lp.to_account_info(),
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