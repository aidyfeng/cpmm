use anchor_lang::{accounts, prelude::*};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{AmmConfig, PoolState, AMM_CONFIG_SEED, POOL_SEED, POOL_VAULT_SEED};

#[derive(Accounts)]
#[instruction(index:u16)]
pub struct Swap<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: pool vault and lp mint authority
    #[account(
        seeds = [
            crate::AUTH_SEED.as_bytes(),
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            AMM_CONFIG_SEED.as_bytes(),
            &index.to_be_bytes()
        ],
        bump = amm_config.bump
    )]
    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// Pool state account
    #[account(
        mut,
        // has_one = lp_mint ,
        // has_one = token_0_vault,
        // has_one = token_1_vault,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            input_token_mint.key().as_ref(),
            output_token_mint.key().as_ref(),
        ],
        bump
    )]
    pub pool_state: Account<'info, PoolState>,

    #[account(
        mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = payer,
        associated_token::token_program = input_token_program
    )]
    pub input_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = output_token_mint,
        associated_token::authority = payer,
        associated_token::token_program = output_token_program
    )]
    pub output_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = input_vault.key() == pool_state.token_0_vault.key() || input_vault.key() == pool_state.token_1_vault.key()
    )]
    pub input_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = output_vault.key() == pool_state.token_0_vault.key() || output_vault.key() == pool_state.token_1_vault.key()
    )]
    pub output_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub input_token_program: Interface<'info, TokenInterface>,

    pub output_token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        address = input_vault.mint
    )]
    pub input_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        address = input_vault.mint
    )]
    pub output_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub associated_program: Program<'info, AssociatedToken>,
}

pub fn swap_base_input(ctx: Context<Swap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
    todo!()
}
