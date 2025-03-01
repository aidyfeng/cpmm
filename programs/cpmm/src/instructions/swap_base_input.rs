use std::ops::DerefMut;

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    error::ErrorCode, pool, transfer_from_pool_vault_to_user, transfer_from_user_to_pool_vault,
    AmmConfig, CurveCalculator, PoolState, TradeDirection, AMM_CONFIG_SEED, POOL_SEED, POOL_VAULT_SEED,
};

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
        bump = pool_state.auth_bump,
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
        constraint = input_vault.key() == pool_state.token_0_vault.key() || input_vault.key() == pool_state.token_1_vault.key(),
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            input_token_mint.key().as_ref()
        ],
        bump,
    )]
    pub input_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = output_vault.key() == pool_state.token_0_vault.key() || output_vault.key() == pool_state.token_1_vault.key(),
        seeds = [
            POOL_VAULT_SEED.as_bytes(),
            pool_state.key().as_ref(),
            output_token_mint.key().as_ref()
        ],
        bump,
    )]
    pub output_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub input_token_program: Interface<'info, TokenInterface>,

    pub output_token_program: Interface<'info, TokenInterface>,

    #[account(
        mut,
        // address = input_vault.mint
    )]
    pub input_token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        // address = input_vault.mint
    )]
    pub output_token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub associated_program: Program<'info, AssociatedToken>,
}

pub fn process_swap_base_input(ctx: Context<Swap>, amount_in: u64, minimum_amount_out: u64) -> Result<()> {
    let block_timestamp = Clock::get()?.unix_timestamp as u64;
    let pool_state = ctx.accounts.pool_state.deref_mut();
    //校验交易池状态及开始时间
    if !pool_state.get_status_by_bit(pool::PoolStatusBitIndex::Swap)
        || block_timestamp < pool_state.open_time
    {
        return err!(ErrorCode::NotApproved);
    }

    require_gt!(amount_in, 0);

    //1.获取两个token vault可交易token及计算价格
    let (
        _trade_direction,
        total_input_token_amount,
        total_output_token_amount,
        _token_0_price_x64,
        _token_1_price_x64,
    ) = if ctx.accounts.input_vault.key() == pool_state.token_0_vault
        && ctx.accounts.output_vault.key() == pool_state.token_1_vault
    {
        let (token_0_price_x64, token_1_price_x64) = pool_state.token_price_x32(
            ctx.accounts.input_vault.amount,
            ctx.accounts.output_vault.amount,
        );

        (
            TradeDirection::ZeroForOne,
            ctx.accounts.input_vault.amount,
            ctx.accounts.output_vault.amount,
            token_0_price_x64,
            token_1_price_x64,
        )
    } else if ctx.accounts.input_vault.key() == pool_state.token_1_vault
        && ctx.accounts.output_vault.key() == pool_state.token_0_vault
    {
        let (token_0_price_x64, token_1_price_x64) = pool_state.token_price_x32(
            ctx.accounts.output_vault.amount,
            ctx.accounts.input_vault.amount,
        );

        (
            TradeDirection::OneForZero,
            ctx.accounts.input_vault.amount,
            ctx.accounts.output_vault.amount,
            token_0_price_x64,
            token_1_price_x64,
        )
    } else {
        return err!(ErrorCode::InvalidVault);
    };

    let constant_before = u128::from(total_input_token_amount)
        .checked_mul(u128::from(total_output_token_amount))
        .unwrap();

    //2.计算可兑换出多少token
    let swap_result = CurveCalculator::swap_base_input(
        u128::from(amount_in),
        u128::from(total_input_token_amount),
        u128::from(total_output_token_amount),
        ctx.accounts.amm_config.trade_fee_rate,
    )
    .ok_or(ErrorCode::ZeroTradingTokens)?;

    let constant_after = swap_result
        .new_swap_source_amount
        .checked_sub(swap_result.trade_fee)
        .unwrap()
        .checked_mul(swap_result.new_swap_destination_amount)
        .unwrap();

    require_eq!(
        u64::try_from(swap_result.source_amount_swapped).unwrap(),
        amount_in
    );

    require_gte!(constant_after, constant_before);

    let amount_out = u64::try_from(swap_result.destination_amount_swapped).unwrap();
    require_gte!(amount_out, minimum_amount_out, ErrorCode::ExceededSlippage);
    //3.transfer token
    //3.1 转移用户amount_in_token到vault
    transfer_from_user_to_pool_vault(
        ctx.accounts.payer.to_account_info(),
        ctx.accounts.input_token_account.to_account_info(),
        ctx.accounts.input_vault.to_account_info(),
        ctx.accounts.input_token_mint.to_account_info(),
        ctx.accounts.input_token_program.to_account_info(),
        amount_in,
        ctx.accounts.input_token_mint.decimals,
    )?;

    //3.2 转移vault token 到 用户destination token account
    transfer_from_pool_vault_to_user(
        ctx.accounts.authority.to_account_info(),
        ctx.accounts.output_vault.to_account_info(),
        ctx.accounts.output_token_account.to_account_info(),
        ctx.accounts.output_token_mint.to_account_info(),
        ctx.accounts.output_token_program.to_account_info(),
        amount_out,
        ctx.accounts.output_token_mint.decimals,
        &[&[crate::AUTH_SEED.as_bytes(), &[pool_state.auth_bump]]],
    )?;

    pool_state.recent_epoch = Clock::get()?.epoch;
    Ok(())
}
