use crate::constants;
use crate::curve::CurveCalculator;
use crate::error::ErrorCode;
use crate::state::*;
use crate::utils::*;
use anchor_lang::{
    accounts::interface_account::InterfaceAccount,
    prelude::*,
    solana_program::clock,
};
use anchor_spl::token::Token;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use std::ops::DerefMut;

#[derive(Accounts)]
#[instruction(index:u16)]
pub struct Initialize<'info> {
    /// Address paying to create the pool. Can be anyone
    #[account(mut)]
    pub creator: Signer<'info>,

    /// Which config the pool belongs to.
    #[account(
        mut,
        seeds = [
            AMM_CONFIG_SEED.as_bytes(),
            &index.to_be_bytes()
        ],
        bump = amm_config.bump
    )]
    pub amm_config: Box<Account<'info, AmmConfig>>,

    /// CHECK: pool vault and lp mint authority
    #[account(
        seeds = [
            crate::AUTH_SEED.as_bytes(),
        ],
        bump,
    )]
    pub authority: UncheckedAccount<'info>,

    /// Initialize an account to store the pool state
    #[account(
        init,
        payer = creator,
        space = constants::DISCRIMINATOR + PoolState::INIT_SPACE,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_0_mint.key().as_ref(),
            token_1_mint.key().as_ref(),
        ],
        bump
    )]
    pub pool_state: Box<Account<'info, PoolState>>,

    /// Token_0 mint, the key must smaller then token_1 mint.
    #[account(
        constraint = token_0_mint.key() < token_1_mint.key(),
        mint::token_program = token_0_program,
    )]
    pub token_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Token_1 mint, the key must grater then token_0 mint.
    #[account(
        mint::token_program = token_1_program,
    )]
    pub token_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// pool lp mint
    #[account(
        init,
        seeds = [
            POOL_LP_MINT_SEED.as_bytes(),
            pool_state.key().as_ref(),
        ],
        bump,
        mint::decimals = 9,
        mint::authority = authority,
        payer = creator,
        mint::token_program = token_program,
    )]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    /// payer token0 account
    #[account(
        mut,
        associated_token::mint = token_0_mint,
        associated_token::authority = creator,
        associated_token::token_program = token_0_program
    )]
    pub creator_token_0: Box<InterfaceAccount<'info, TokenAccount>>,

    /// creator token1 account
    #[account(
        mut,
        associated_token::mint = token_1_mint,
        associated_token::authority = creator,
        associated_token::token_program = token_1_program
    )]
    pub creator_token_1: Box<InterfaceAccount<'info, TokenAccount>>,

    /// creator lp token account
    #[account(
        init,
        associated_token::mint = lp_mint,
        associated_token::authority = creator,
        payer = creator,
        associated_token::token_program = token_program,
    )]
    pub creator_lp_token: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Token_0 vault for the pool, created by contract
    #[account(
        init,
        payer = creator,
        token::mint = token_0_mint,
        token::authority = authority,
        token::token_program = token_0_program,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_0_mint.key().as_ref()
        ],
        bump,
    )]
    pub token_0_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK: Token_1 vault for the pool, created by contract
    #[account(
        init,
        payer = creator,
        token::mint = token_1_mint,
        token::authority = authority,
        token::token_program = token_1_program,
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            token_1_mint.key().as_ref()
        ],
        bump,
    )]
    pub token_1_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    pub token_0_program: Interface<'info, TokenInterface>,
    pub token_1_program: Interface<'info, TokenInterface>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// To create a new program account
    pub system_program: Program<'info, System>,
    /* /// Sysvar for program account
    pub rent: Sysvar<'info, Rent>, */
}

pub fn process_initialize(
    ctx: Context<Initialize>,
    init_amount_0: u64,
    init_amount_1: u64,
    mut open_time: u64,
) -> Result<()> {
    //1.判断mint是否被支持
    if !(is_supported_mint(&ctx.accounts.token_0_mint).unwrap()
        && is_supported_mint(&ctx.accounts.token_1_mint).unwrap())
    {
        return err!(ErrorCode::NotSupportMint);
    }

    if ctx.accounts.amm_config.disable_create_pool {
        return err!(ErrorCode::NotApproved);
    }
    let block_timestamp = clock::Clock::get()?.unix_timestamp as u64;
    if open_time <= block_timestamp {
        open_time = block_timestamp + 1;
    }

    let pool_state = ctx.accounts.pool_state.deref_mut();

    //2.user account transfer to vault account
     transfer_from_user_to_pool_vault(
        ctx.accounts.creator.to_account_info(),
        ctx.accounts.creator_token_0.to_account_info(),
        ctx.accounts.token_0_vault.to_account_info(),
        ctx.accounts.token_0_mint.to_account_info(),
        ctx.accounts.token_0_program.to_account_info(),
        init_amount_0,
        ctx.accounts.token_0_mint.decimals,
    )?;

    transfer_from_user_to_pool_vault(
        ctx.accounts.creator.to_account_info(),
        ctx.accounts.creator_token_1.to_account_info(),
        ctx.accounts.token_1_vault.to_account_info(),
        ctx.accounts.token_1_mint.to_account_info(),
        ctx.accounts.token_1_program.to_account_info(),
        init_amount_1,
        ctx.accounts.token_1_mint.decimals,
    )?;

    ctx.accounts.token_0_vault.reload()?;
    ctx.accounts.token_1_vault.reload()?;

    CurveCalculator::validate_supply(ctx.accounts.token_0_vault.amount, ctx.accounts.token_1_vault.amount)?;

    //3.计算liquidity,并mint_to user_lp_token_amount
    let liquidity = U128::from(ctx.accounts.token_0_vault.amount)
        .checked_mul(ctx.accounts.token_1_vault.amount.into())
        .unwrap()
        .integer_sqrt()
        .as_u64();
    let lock_lp_amount = 100;
    msg!(
        "liquidity:{}, lock_lp_amount:{}, vault_0_amount:{},vault_1_amount:{}",
        liquidity,
        lock_lp_amount,
        ctx.accounts.token_0_vault.amount,
        ctx.accounts.token_1_vault.amount
    );
    token::token_mint_to(
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.token_program.to_account_info(),
        ctx.accounts.lp_mint.to_account_info(),
        ctx.accounts.creator_lp_token.to_account_info(),
        liquidity
            .checked_sub(lock_lp_amount)
            .ok_or(ErrorCode::InitLpAmountTooLess)?,
        &[&[crate::AUTH_SEED.as_bytes(), &[ctx.bumps.authority]]],
    )?;

    //4.初始化pool_state账户
    pool_state.initialize(
        ctx.bumps.authority,
        liquidity,
        open_time,
        ctx.accounts.creator.key(),
        ctx.accounts.amm_config.key(),
        ctx.accounts.token_0_vault.key(),
        ctx.accounts.token_1_vault.key(),
        &ctx.accounts.token_0_mint,
        &ctx.accounts.token_1_mint,
        &ctx.accounts.lp_mint,
        ctx.bumps.pool_state,
    ); 
    Ok(())
}
