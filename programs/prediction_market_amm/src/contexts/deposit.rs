use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked, MintTo, mint_to}};

use crate::{assert_non_zero, assert_not_expired, assert_not_locked, states:: Market};

#[derive(Accounts)]
pub struct Deposit<'info> {
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
        mut,
        mint::token_program = token_program,
    )]
    mint_usdc: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        seeds = [b"lp", market.key().as_ref()],
        bump,
        payer = user,
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
        associated_token::authority = market,
        associated_token::token_program = token_program
    )]
    vault_usdc: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_usdc,
        associated_token::authority = user,
        associated_token::token_program = token_program
    )]
    user_ata_usdc: Box<InterfaceAccount<'info, TokenAccount>>,
    // #[account(
    //     mut,
    //     associated_token::mint = mint_yes,
    //     associated_token::authority = user,
    //     associated_token::token_program = token_program
    // )]
    // user_ata_yes: Box<InterfaceAccount<'info, TokenAccount>>,
    // #[account(
    //     mut,
    //     associated_token::mint = mint_no,
    //     associated_token::authority = user,
    //     associated_token::token_program = token_program
    // )]
    // user_ata_no: Box<InterfaceAccount<'info, TokenAccount>>,
    #[account(
        init,
        payer = user,
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

impl<'info> Deposit<'info> {
    pub fn deposit(
        &mut self,
        usdc_amount: u64,
        min_no: u64,
        min_yes: u64,
        expiration: i64,
    ) -> Result<()> {
        assert_not_locked!(self.market.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([usdc_amount, min_yes, min_no]);

        let cpi_account = TransferChecked {
            from: self.user_ata_usdc.to_account_info(),
            mint: self.mint_usdc.to_account_info(),
            to: self.vault_usdc.to_account_info(), 
            authority: self.user.to_account_info()
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_account);

        
        transfer_checked(ctx, usdc_amount, self.mint_usdc.decimals)?;

        let yes_amount = usdc_amount;
        let no_amount = usdc_amount;
        
        msg!("yes amount is {:?}", yes_amount);
        msg!("no amount is {:?}", yes_amount);
        msg!("min yes amount is {:?}", min_yes);
        msg!("min no amount is {:?}", min_no);

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
        let ( to, mint) = match is_yes {
            true => (self.vault_yes.to_account_info(), self.mint_yes.to_account_info()),
            false => (self.vault_no.to_account_info(), self.mint_no.to_account_info())
        };
        
        let cpi_account = MintTo {
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

        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), cpi_account, signer_seeds);
        
        mint_to(ctx, amount)
    }

    pub fn mint_lp_tokens(
        &self,
        amount: u64,
    ) -> Result<()> {
        msg!("mint authority {:?}", self.mint_lp.mint_authority);
        let accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_ata_lp.to_account_info(),
            authority: self.market.to_account_info(), 
        };

        let seeds = &[
            &b"market"[..],
            &self.market.seed.to_le_bytes(),
            &[self.market.market_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, signer_seeds);

        mint_to(ctx, amount)
    }
}