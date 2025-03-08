use std::ops::DerefMut;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::Token, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{error::ErrorCode, token_mint_to, transfer_from_user_to_pool_vault, AmmConfig, CurveCalculator, PoolState, AMM_CONFIG_SEED, AUTH_SEED, POOL_SEED};


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
        bump = pool_state.auth_bump
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
    pub amm_config:Box<Account<'info,AmmConfig>>,

    /// pool state
    #[account(
        mut,
        has_one = lp_mint @ ErrorCode::NotApproved,
        // has_one = token_0_mint,
        // has_one = token_1_mint,
        has_one = token_0_vault @ ErrorCode::NotApproved,
        has_one = token_1_vault @ ErrorCode::NotApproved,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_0_mint.key().as_ref(),
            token_1_mint.key().as_ref(),
        ],
        bump = pool_state.bump
    )]
    pub pool_state:Box<Account<'info,PoolState>>,

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
    pub owner_lp_token:Box<InterfaceAccount<'info,TokenAccount>>,

    /// the mint of token 0
    #[account(
        mut,
        address = token_0_account.mint @ ErrorCode::NotApproved
    )]
    pub token_0_mint: Box<InterfaceAccount<'info,Mint>>,

    /// owner's account of token 0
    #[account(
        mut,
        associated_token::mint = token_0_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_0_program
    )]
    pub token_0_account: Box<InterfaceAccount<'info,TokenAccount>>,

    /// the mint of token 1
    #[account(
        mut,
        address = token_1_account.mint @ ErrorCode::NotApproved
    )]
    pub token_1_mint: Box<InterfaceAccount<'info,Mint>>,

    /// owner's account of token 1
    #[account(
        mut,
        associated_token::mint = token_1_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_1_program
    )]
    pub token_1_account: Box<InterfaceAccount<'info,TokenAccount>>,


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
    pub token_0_program: Interface<'info, TokenInterface>,

    pub token_1_program: Interface<'info, TokenInterface>,

    pub token_program:Program<'info,Token>,

    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,
}


pub fn process_deposit(
    ctx: Context<Deposit>,
    lp_token_amount: u64,
    maximum_token_0_amount: u64,
    maximum_token_1_amount: u64,
) -> Result<()> {

    let pool_state = ctx.accounts.pool_state.deref_mut();

    //1.检查 pool state
    if !pool_state.get_status_by_bit(crate::PoolStatusBitIndex::Deposit) {
        return err!(ErrorCode::NotApproved);
    }
    
    //2.计算交易tokens
    let result = CurveCalculator::lp_tokens_to_trading_tokens(
            u128::from(lp_token_amount), 
            u128::from(pool_state.lp_supply), 
            u128::from(ctx.accounts.token_0_vault.amount), 
            u128::from(ctx.accounts.token_1_vault.amount), 
            crate::RoundDirection::Ceiling
        ).ok_or(ErrorCode::ZeroTradingTokens)?;

    let token_0_amount = u64::try_from(result.token_0_amount).unwrap();
    let token_1_amount = u64::try_from(result.token_1_amount).unwrap();

    if token_0_amount > maximum_token_0_amount 
        || token_1_amount > maximum_token_1_amount {
            return Err(ErrorCode::ExceededSlippage.into());
    }

    //3. 把 user token 转到 vault账户
    //3.1 转账token_0
    transfer_from_user_to_pool_vault(
        ctx.accounts.owner.to_account_info(), 
        ctx.accounts.token_0_account.to_account_info(), 
        ctx.accounts.token_0_vault.to_account_info(), 
        ctx.accounts.token_0_mint.to_account_info(), 
        ctx.accounts.token_0_program.to_account_info(),
        token_0_amount, 
        ctx.accounts.token_0_mint.decimals)?;

    //3.2 转账token_1
    transfer_from_user_to_pool_vault(
        ctx.accounts.owner.to_account_info(), 
        ctx.accounts.token_1_account.to_account_info(), 
        ctx.accounts.token_1_vault.to_account_info(), 
        ctx.accounts.token_1_mint.to_account_info(), 
        ctx.accounts.token_1_program.to_account_info(),
        token_1_amount, 
        ctx.accounts.token_1_mint.decimals)?;
    
    //4.更新pool_state
    pool_state.lp_supply = pool_state.lp_supply.checked_add(lp_token_amount).unwrap();

    //5.mint lp_tokens
    token_mint_to(
    ctx.accounts.authority.to_account_info(),
    ctx.accounts.token_program.to_account_info(),
    ctx.accounts.lp_mint.to_account_info(),
    ctx.accounts.owner_lp_token.to_account_info(), 
    lp_token_amount, 
    &[&[AUTH_SEED.as_bytes(),&[pool_state.auth_bump]]])?;

    pool_state.recent_epoch = Clock::get()?.epoch;

    Ok(())
}