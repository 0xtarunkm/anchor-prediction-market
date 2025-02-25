use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{
        burn, transfer_checked, Burn, Mint, TokenAccount, TokenInterface, TransferChecked,
    },
};

use crate::{assert_not_locked, error::MarketError, states::Market};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    user: Signer<'info>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = user
    )]
    mint_yes: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        mint::token_program = token_program,
        mint::authority = user
    )]
    mint_no: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mint::token_program = token_program,
    )]
    mint_usdc: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        mut,
        associated_token::mint = mint_yes,
        associated_token::authority = market,
    )]
    vault_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_no,
        associated_token::authority = market,
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
        associated_token::mint = mint_usdc,
        associated_token::authority = user,
    )]
    user_ata_usdc: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        has_one = mint_yes,
        has_one = mint_no,
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

        require!(!self.market.settled, MarketError::MarketNotSettled);

        let (user_tokens, total_tokens) = if is_yes {
            (self.user_ata_yes.amount, self.mint_yes.supply)
        } else {
            (self.user_ata_no.amount, self.mint_no.supply)
        };

        require!(user_tokens > 0, MarketError::InsufficientBalance);

        let total_payout = self.vault_usdc.amount;

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
            from: self.vault_usdc.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.user_ata_usdc.to_account_info(),
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
            accounts,
            signer_seeds,
        );

        transfer_checked(ctx, amount, self.mint_usdc.decimals)
    }

    pub fn burn_tokens(&self, amount: u64, is_yes: bool) -> Result<()> {
        let (mint, from) = match is_yes {
            true => (
                self.mint_yes.to_account_info(),
                self.user_ata_yes.to_account_info(),
            ),
            false => (
                self.mint_no.to_account_info(),
                self.user_ata_no.to_account_info(),
            ),
        };

        let cpi_accounts = Burn {
            mint,
            from,
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

        burn(ctx, amount)
    }
}
