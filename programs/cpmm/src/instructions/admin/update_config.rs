use crate::curve::fees::FEE_RATE_DENOMINATOR_VALUE;
use crate::error::ErrorCode;
use crate::state::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(index:u16)]
pub struct UpdateAmmConfig<'info> {
    /// The amm config owner or admin
    // #[account(address = crate::admin::id() @ ErrorCode::InvalidOwner)]
    pub owner: Signer<'info>,

    /// Amm config account to be changed
    #[account(
        mut,
        seeds = [
            AMM_CONFIG_SEED.as_bytes(),
            &index.to_be_bytes()
        ],
        bump = amm_config.bump
    )]
    pub amm_config: Account<'info, AmmConfig>,
}

pub fn process_update_amm_config(
    ctx: Context<UpdateAmmConfig>,
    param: u8,
    value: u64,
    _index: u16,
) -> Result<()> {
    let amm_config: &mut Account<'_, AmmConfig> = &mut ctx.accounts.amm_config;
    let match_param: Option<u8> = Some(param);
    match match_param {
        Some(0) => update_trade_fee_rate(amm_config, value),
        Some(1) => amm_config.disable_create_pool = if value == 0 { false } else { true },
        _ => return err!(ErrorCode::InvalidInput),
    }

    Ok(())
}

fn update_trade_fee_rate(amm_config: &mut Account<AmmConfig>, trade_fee_rate: u64) {
    assert!(trade_fee_rate < FEE_RATE_DENOMINATOR_VALUE);
    amm_config.trade_fee_rate = trade_fee_rate;
}

