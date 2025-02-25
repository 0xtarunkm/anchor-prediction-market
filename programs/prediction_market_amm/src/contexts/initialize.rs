use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::{create_idempotent, AssociatedToken},
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata,
    },
    token_interface::{Mint, TokenInterface},
};

use crate::states::Market;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    signer: Signer<'info>,
    #[account(
        init,
        seeds = [b"yes_mint", seed.to_le_bytes().as_ref()],
        bump,
        payer = signer,
        mint::token_program = token_program,
        mint::authority = market,
        mint::decimals = 6
    )]
    mint_yes: Box<InterfaceAccount<'info, Mint>>,
    #[account(
        init,
        seeds = [b"no_mint", seed.to_le_bytes().as_ref()],
        bump,
        payer = signer,
        mint::token_program = token_program,
        mint::authority = market,
        mint::decimals = 6
    )]
    mint_no: Box<InterfaceAccount<'info, Mint>>,
    mint_usdc: Box<InterfaceAccount<'info, Mint>>,
    /// CHECK: This account is not read or written in this instruction
    #[account(mut)]
    vault_yes: UncheckedAccount<'info>,
    /// CHECK: This account is not read or written in this instruction
    #[account(mut)]
    vault_no: UncheckedAccount<'info>,
    /// CHECK: This account is not read or written in this instruction
    #[account(mut)]
    vault_usdc: UncheckedAccount<'info>,
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    metadata_yes: UncheckedAccount<'info>,
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    metadata_no: UncheckedAccount<'info>,
    #[account(
        init,
        payer = signer,
        seeds = [b"market", seed.to_le_bytes().as_ref()],
        bump,
        space = 8 + Market::INIT_SPACE
    )]
    market: Box<Account<'info, Market>>,

    system_program: Program<'info, System>,
    token_program: Interface<'info, TokenInterface>,
    token_metadata_program: Program<'info, Metadata>,
    associated_token_program: Program<'info, AssociatedToken>,
    rent: Sysvar<'info, Rent>,
}

impl<'info> Initialize<'info> {
    pub fn save_market(
        &mut self,
        seed: u64,
        name: String,
        token_yes_name: String,
        token_yes_symbol: String,
        token_no_name: String,
        token_no_symbol: String,
        token_yes_uri: String,
        token_no_uri: String,
        fee: u16,
        end_time: i64,
        bumps: &InitializeBumps,
    ) -> Result<()> {
        self.market.set_inner(Market {
            market_name: name,
            seed,
            mint_yes: self.mint_yes.key(),
            mint_no: self.mint_no.key(),
            total_liquidity: 0,
            end_time,
            fee,
            locked: false,
            settled: false,
            market_bump: bumps.market,
        });

        create_idempotent(CpiContext::new(
            self.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: self.signer.to_account_info(),
                associated_token: self.vault_usdc.to_account_info(),
                authority: self.market.to_account_info(),
                mint: self.mint_usdc.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        ))?;
        create_idempotent(CpiContext::new(
            self.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: self.signer.to_account_info(),
                associated_token: self.vault_yes.to_account_info(),
                authority: self.market.to_account_info(),
                mint: self.mint_yes.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        ))?;
        create_idempotent(CpiContext::new(
            self.associated_token_program.to_account_info(),
            anchor_spl::associated_token::Create {
                payer: self.signer.to_account_info(),
                associated_token: self.vault_no.to_account_info(),
                authority: self.market.to_account_info(),
                mint: self.mint_no.to_account_info(),
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        ))?;

        let token_yes_data = DataV2 {
            name: token_yes_name,
            symbol: token_yes_symbol,
            uri: token_yes_uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let token_no_data = DataV2 {
            name: token_no_name,
            symbol: token_no_symbol,
            uri: token_no_uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let seeds = &[
            &b"market"[..],
            &self.market.seed.to_le_bytes(),
            &[self.market.market_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let metadata_yes_ctx = CpiContext::new_with_signer(
            self.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: self.signer.to_account_info(),
                update_authority: self.market.to_account_info(),
                mint: self.mint_yes.to_account_info(),
                metadata: self.metadata_yes.to_account_info(),
                mint_authority: self.market.to_account_info(),
                system_program: self.system_program.to_account_info(),
                rent: self.rent.to_account_info(),
            },
            signer_seeds,
        );

        create_metadata_accounts_v3(metadata_yes_ctx, token_yes_data, false, true, None)?;

        let seeds = &[
            &b"market"[..],
            &self.market.seed.to_le_bytes(),
            &[self.market.market_bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let metadata_no_ctx = CpiContext::new_with_signer(
            self.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                payer: self.signer.to_account_info(),
                update_authority: self.market.to_account_info(),
                mint: self.mint_no.to_account_info(),
                metadata: self.metadata_no.to_account_info(),
                mint_authority: self.market.to_account_info(),
                system_program: self.system_program.to_account_info(),
                rent: self.rent.to_account_info(),
            },
            signer_seeds,
        );

        create_metadata_accounts_v3(metadata_no_ctx, token_no_data, false, true, None)?;

        Ok(())
    }
}
