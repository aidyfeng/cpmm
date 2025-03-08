use std::ops::DerefMut;

use crate::state::*;
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

#[derive(Accounts)]
#[instruction(index:u16)]
pub struct UpdatePoolStatus<'info> {
    #[account(
        mut
        // address = crate::admin::id()
    )]
    pub authority: Signer<'info>,

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

    #[account(
        mut,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_0_mint.key().as_ref(),
            token_1_mint.key().as_ref(),
        ],
        bump = pool_state.bump
    )]
    pub pool_state: Box<Account<'info, PoolState>>,


    /// Token_0 mint, the key must smaller then token_1 mint.
    #[account(
        constraint = token_0_mint.key() < token_1_mint.key(),
        // mint::token_program = token_0_program,
    )]
    pub token_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Token_1 mint, the key must grater then token_0 mint.
    #[account(
        // mint::token_program = token_1_program,
    )]
    pub token_1_mint: Box<InterfaceAccount<'info, Mint>>,
}

pub fn process_update_pool_status(ctx: Context<UpdatePoolStatus>, status: u8) -> Result<()> {
    require_gte!(255, status);
    let pool_state =ctx.accounts.pool_state.deref_mut();
    pool_state.set_status(status);
    pool_state.recent_epoch = Clock::get()?.epoch;
    Ok(())
}
