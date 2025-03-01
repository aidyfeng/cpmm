use std::ops::DerefMut;

use anchor_lang::prelude::*;

use crate::{
    error::ErrorCode, transfer_from_pool_vault_to_user,
    transfer_from_user_to_pool_vault, CurveCalculator, PoolStatusBitIndex, TradeDirection,
};

use super::Swap;

pub fn process_swap_base_output(
    ctx: Context<Swap>,
    amount_out: u64,
    max_amount_in: u64,
) -> Result<()> {
    let block_timestamp = Clock::get()?.unix_timestamp as u64;
    let pool_state = ctx.accounts.pool_state.deref_mut();

    //校验交易池状态及开始时间
    if !pool_state.get_status_by_bit(PoolStatusBitIndex::Swap)
        || block_timestamp < pool_state.open_time
    {
        return err!(ErrorCode::NotApproved);
    }

    require_gt!(amount_out, 0);

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

    //2.计算需要投入多少token
    let swap_result = CurveCalculator::swap_base_output(
        u128::from(amount_out),
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
        u64::try_from(swap_result.destination_amount_swapped).unwrap(),
        amount_out
    );

    require_gte!(constant_after, constant_before);

    let amount_in: u64 = u64::try_from(swap_result.source_amount_swapped).unwrap();
    require_gte!(max_amount_in, amount_in, ErrorCode::ExceededSlippage);

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
