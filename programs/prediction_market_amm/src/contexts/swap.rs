use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{
    assert_non_zero, assert_not_expired, assert_not_locked, error::MarketError,
    helpers::calculate_lmsr_output, states::Market,
};

#[derive(Accounts)]
pub struct Swap<'info> {
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
        init_if_needed,
        payer = user,
        associated_token::mint = mint_yes,
        associated_token::authority = user,
    )]
    user_ata_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = user,
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

impl<'info> Swap<'info> {
    pub fn swap(
        &mut self,
        is_buying: bool,
        amount_in: u64,
        is_yes: bool,
        min_out: u64,
        expiration: i64,
    ) -> Result<()> {
        assert_not_locked!(self.market.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([amount_in, min_out]);

        let amount_out = calculate_lmsr_output(
            amount_in,
            self.vault_yes.amount,
            self.vault_no.amount,
            is_buying,
            is_yes,
        )?;

        require!(amount_out >= min_out, MarketError::SlippageExceeded);

        if is_buying {
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
        amount: u64,
    ) -> Result<()> {
        let (mint, from, to, decimals) = match is_usdc {
            true => (
                self.mint_usdc.to_account_info(),
                self.user_ata_usdc.to_account_info(),
                self.vault_usdc.to_account_info(),
                self.mint_usdc.decimals,
            ),
            false => match is_yes {
                Some(true) => (
                    self.mint_yes.to_account_info(),
                    self.user_ata_yes.to_account_info(),
                    self.vault_yes.to_account_info(),
                    self.mint_yes.decimals,
                ),
                Some(false) => (
                    self.mint_no.to_account_info(),
                    self.user_ata_no.to_account_info(),
                    self.vault_no.to_account_info(),
                    self.mint_no.decimals,
                ),
                None => return Err(MarketError::InvalidToken.into()),
            },
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
        is_yes: Option<bool>,
    ) -> Result<()> {
        let (mint, from, to, decimals) = match is_usdc {
            true => (
                self.mint_usdc.to_account_info(),
                self.vault_usdc.to_account_info(),
                self.user_ata_usdc.to_account_info(),
                self.mint_usdc.decimals,
            ),
            false => match is_yes {
                Some(true) => (
                    self.mint_yes.to_account_info(),
                    self.vault_yes.to_account_info(),
                    self.user_ata_yes.to_account_info(),
                    self.mint_yes.decimals,
                ),
                Some(false) => (
                    self.mint_no.to_account_info(),
                    self.vault_no.to_account_info(),
                    self.user_ata_no.to_account_info(),
                    self.mint_no.decimals,
                ),
                None => return Err(MarketError::InvalidToken.into()),
            },
        };

        let account = TransferChecked {
            from,
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
            account,
            signer_seeds,
        );

        transfer_checked(ctx, amount, decimals)
    }
}
