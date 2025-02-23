use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::Token, token_interface::{Mint, TokenAccount}};

use crate::{AmmConfig, PoolState, POOL_SEED,AMM_CONFIG_SEED};


#[derive(Accounts)]
#[instruction(index:u16)]
pub struct Deposit<'info>{

    /// Pays to mint the position
    #[account(mut)]
    pub owner:Signer<'info>,

    /// CHECK: pool vault and lp mint authority
    #[account(
        seeds = [
            crate::AUTH_SEED.as_bytes(),
        ],
        bump 
    )]
    pub authority:UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            AMM_CONFIG_SEED.as_bytes(),
            &index.to_be_bytes()
        ],
        bump = amm_config.bump
    )]
    pub amm_config:Account<'info,AmmConfig>,

    /// pool state
    #[account(
        has_one = lp_mint,
        has_one = token_0_mint,
        has_one = token_1_mint,
        has_one = token_0_vault,
        has_one = token_1_vault,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_0_mint.key().as_ref(),
            token_1_mint.key().as_ref(),
        ],
        bump
    )]
    pub pool_state:Account<'info,PoolState>,

    /// Lp token mint
    #[account(mut)]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    /// owner Lp token account
    #[account(
        init_if_needed,
        associated_token::mint = lp_mint,
        associated_token::authority = owner,
        payer = owner,
        associated_token::token_program = token_program,
    )]
    pub owner_lp_token:InterfaceAccount<'info,TokenAccount>,

    /// the mint of token 0
    #[account(
        mut
    )]
    pub token_0_mint: InterfaceAccount<'info,Mint>,

    /// owner's account of token 0
    #[account(
        mut,
        associated_token::mint = token_0_mint,
        associated_token::authority = owner,
    )]
    pub token_0_account: InterfaceAccount<'info,TokenAccount>,

    /// the mint of token 1
    #[account(
        mut
    )]
    pub token_1_mint: InterfaceAccount<'info,Mint>,

    /// owner's account of token 1
    #[account(
        mut,
        associated_token::mint = token_1_mint,
        associated_token::authority = owner,
    )]
    pub token_1_account: InterfaceAccount<'info,TokenAccount>,


    /// The address that holds pool tokens for token_0
    #[account(
        mut
    )]
    pub token_0_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_1
    #[account(
        mut
    )]
    pub token_1_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// the system program
    pub system_program: Program<'info,System>,

    /// the token program
    pub token_program: Program<'info, Token>,

    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
}