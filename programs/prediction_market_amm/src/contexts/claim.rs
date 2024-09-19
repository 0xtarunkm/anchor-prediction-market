use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{burn, Burn, Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{assert_not_locked, error::MarketError, states::Market};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
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
    user_ata_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_no,
        associated_token::authority = user,
    )]
    user_ata_no: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_stablecoin,
        associated_token::authority = user,
    )]
    user_ata_stablecoin: Box<InterfaceAccount<'info, TokenAccount>>,
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

impl<'info> ClaimReward<'info> {
    pub fn claim(&mut self, is_yes: bool) -> Result<()> {
        assert_not_locked!(self.market.locked);

        require!(self.market.settled, MarketError::MarketAlreadySettled);

        let (user_tokens, total_tokens) = if is_yes {
            (self.user_ata_yes.amount, self.mint_yes.supply)
        } else {
            (self.user_ata_no.amount, self.mint_no.supply)
        };

        require!(user_tokens > 0, MarketError::InsufficientBalance);

        let total_payout = self.vault_stablecoin.amount;

        let user_payout = (user_tokens as u128)
            .checked_mul(total_payout as u128)
            .unwrap()
            .checked_div(total_tokens as u128)
            .unwrap() as u64;

        self.transfer_amount(user_payout)?;

        self.burn_tokens(user_tokens, is_yes)
    }

    pub fn transfer_amount(&self, amount: u64) -> Result<()> {
        let accounts = TransferChecked {
            from: self.vault_stablecoin.to_account_info(),
            mint: self.mint_stablecoin.to_account_info(),
            to: self.user_ata_stablecoin.to_account_info(),
            authority: self.auth.to_account_info()
        };

        let seeds = &[
            &b"auth"[..],
            &[self.market.auth_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, signer_seeds);

        transfer_checked(ctx, amount, self.mint_stablecoin.decimals)
    }

    pub fn burn_tokens(&self, amount: u64, is_yes: bool) -> Result<()> {
        let (mint, from) = match is_yes {
            true => (self.mint_yes.to_account_info(), self.user_ata_yes.to_account_info()),
            false => (self.mint_no.to_account_info(), self.user_ata_no.to_account_info())
        };

        let cpi_accounts = Burn {
            mint,
            from,
            authority: self.user.to_account_info()
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        burn(ctx, amount)
    }
}