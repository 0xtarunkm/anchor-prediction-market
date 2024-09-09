use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};
use prediction_market_curve::OutcomeToken;

use crate::states::Market;

#[derive(Accounts)]
pub struct Swap<'info> {
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
    pub market: Account<'info, Market>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Swap<'info> {
    pub fn deposit_tokens(
        &mut self,
        token: OutcomeToken,
        amount: u64,
    ) -> Result<()> {
        let mint;
        let (from, to) = match token {
            OutcomeToken::YES => {
                mint = self.mint_yes.clone();
                (self.user_stablecoin.to_account_info(), self.vault_yes.to_account_info())
            },
            OutcomeToken::NO => {
                mint = self.mint_no.clone();
                (self.user_stablecoin.to_account_info(), self.vault_no.to_account_info())
            }
        };

        let account = TransferChecked {
            from,
            mint: mint.to_account_info(),
            to,
            authority: self.user.to_account_info()
        };
        let ctx = CpiContext::new(self.token_program.to_account_info(), account);

        transfer_checked(ctx, amount, 6)
    }

    pub fn withdraw_token(
        &mut self,
        token: OutcomeToken,
        amount: u64,
    ) -> Result<()> {
        let mint;
        let (from, to) = match token {
            OutcomeToken::YES => {
                mint = self.mint_yes.clone();
                (self.vault_yes.to_account_info(), self.user_yes.to_account_info())
            }, 
            OutcomeToken::NO => {
                mint = self.mint_no.clone();
                (self.vault_no.to_account_info(), self.user_no.to_account_info())
            }
        };

        let account = TransferChecked {
            from,
            mint: mint.to_account_info(),
            to,
            authority: self.auth.to_account_info()
        };

        let seeds = &[
            &b"auth"[..],
            &[self.market.auth_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), account, signer_seeds);

        transfer_checked(ctx, amount, 6)
    }
}