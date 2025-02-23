use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;

use crate::{AmmConfig, PoolState, POOL_SEED};


#[derive(Accounts)]
pub struct Deposit<'info>{

    pub owner:Signer<'info>,

    #[account(
        seeds = [
            crate::AUTH_SEED.as_bytes(),
        ],
        bump 
    )]
    pub authority:UncheckedAccount<'info>,

    pub amm_config:Account<'info,AmmConfig>,

    #[account(
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            token_0_mint.key().as_ref(),
            token_1_mint.key().as_ref(),
        ],
        bump
    )]
    pub pool_state:Account<'info,PoolState>,

    pub token_0_mint: InterfaceAccount<'info,Mint>,

    pub token_1_mint: InterfaceAccount<'info,Mint>
}