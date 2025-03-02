use std::ops::DerefMut;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::Token, token_interface::{Mint, TokenAccount, TokenInterface}};
use crate::{error::ErrorCode, token_burn, transfer_from_pool_vault_to_user, AmmConfig, CurveCalculator, PoolState, AMM_CONFIG_SEED, POOL_SEED};

#[derive(Accounts)]
#[instruction(index:u16)]
pub struct Withdraw<'info> {
    #[account(mut)]
    /// Pays to mint the position
    pub owner: Signer<'info>,

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
    pub amm_config:Box<Account<'info,AmmConfig>>,

    /// Pool state account
    #[account(
        mut,
        has_one = lp_mint ,
        has_one = token_0_vault,
        has_one = token_1_vault,
        seeds = [
            POOL_SEED.as_bytes(),
            amm_config.key().as_ref(),
            vault_0_mint.key().as_ref(),
            vault_1_mint.key().as_ref(),
        ],
        bump
    )]
    pub pool_state: Account<'info, PoolState>,

    /// Owner lp token account
    #[account(
        mut, 
        associated_token::mint = lp_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_program,
    )]
    pub owner_lp_token: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The token account for receive token_0, 
    #[account(
        mut,
        associated_token::mint = vault_0_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_0_program
    )]
    pub token_0_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The token account for receive token_1
    #[account(
        mut,
        associated_token::mint = vault_1_mint,
        associated_token::authority = owner,
        associated_token::token_program = token_1_program
    )]
    pub token_1_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_0
    #[account(
        mut,
    )]
    pub token_0_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_1
    #[account(
        mut,
    )]
    pub token_1_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// token_0_program
    pub token_0_program: Interface<'info,TokenInterface>,

    /// token_1_program
    pub token_1_program: Interface<'info, TokenInterface>,

    pub token_program:Program<'info,Token>,

    /// The mint of token_0 vault
    #[account(
        address = token_0_vault.mint
    )]
    pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token_1 vault
    #[account(
        address = token_1_vault.mint
    )]
    pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Pool lp token mint
    #[account(
        mut,
    )]
    pub lp_mint: Box<InterfaceAccount<'info, Mint>>,

    /// memo program
    /// CHECK:
    /* #[account(
        address = spl_memo::id()
    )]
    pub memo_program: UncheckedAccount<'info>, */

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program : Program<'info,System>
}

pub fn process_withdraw(
    ctx:Context<Withdraw>,
    lp_token_amount:u64,
    minimum_token_0_amount:u64,
    minimum_token_1_amount:u64
    ) -> Result<()>{
        require_gt!(lp_token_amount,0);
        let pool_state = ctx.accounts.pool_state.deref_mut();
        if !pool_state.get_status_by_bit(crate::PoolStatusBitIndex::Withdraw) {
            return err!(ErrorCode::NotApproved);
        }
        let total_token_0_amount = ctx.accounts.token_0_vault.amount;
        let total_token_1_amount = ctx.accounts.token_1_vault.amount;

        //1.计算需要提取的token_1_amount, 和token_2_amount
        let result = CurveCalculator::lp_tokens_to_trading_tokens(
                    u128::from(lp_token_amount), 
                    u128::from(pool_state.lp_supply), 
                    u128::from(total_token_0_amount), 
                    u128::from(total_token_1_amount), 
                    crate::RoundDirection::Ceiling
                ).ok_or(ErrorCode::ZeroTradingTokens)?;

        let token_0_amount = std::cmp::min(total_token_0_amount, u64::try_from(result.token_0_amount).unwrap());
        let token_1_amount = std::cmp::min(total_token_1_amount,u64::try_from(result.token_1_amount).unwrap());
        //2.校验是否超过最大滑点
        if token_0_amount < minimum_token_0_amount 
                || token_1_amount < minimum_token_1_amount {
            return err!(ErrorCode::ExceededSlippage);
        }

        pool_state.lp_supply = pool_state.lp_supply.checked_sub(lp_token_amount).unwrap();

        //3.burn lp_tokens
        token_burn(
            ctx.accounts.owner.to_account_info(), 
            ctx.accounts.token_program.to_account_info(), 
            ctx.accounts.lp_mint.to_account_info(), 
            ctx.accounts.owner_lp_token.to_account_info(), 
            lp_token_amount, 
            &[&[crate::AUTH_SEED.as_bytes(),&[ctx.bumps.authority]]])?;

        //4.从vault 转账到user_token_account
        //4.1 从token_0_vault 转到 token_0_account
        transfer_from_pool_vault_to_user(
            ctx.accounts.authority.to_account_info(), 
            ctx.accounts.token_0_vault.to_account_info(), 
            ctx.accounts.token_0_account.to_account_info(), 
            ctx.accounts.vault_0_mint.to_account_info(), 
            ctx.accounts.token_0_program.to_account_info(), 
            token_0_amount, 
            ctx.accounts.vault_0_mint.decimals, 
            &[&[crate::AUTH_SEED.as_bytes(),&[ctx.bumps.authority]]]
        )?;

        //4.2 从token_1_vault 转到 token_1_account
        transfer_from_pool_vault_to_user(
            ctx.accounts.authority.to_account_info(), 
            ctx.accounts.token_1_vault.to_account_info(), 
            ctx.accounts.token_1_account.to_account_info(), 
            ctx.accounts.vault_1_mint.to_account_info(), 
            ctx.accounts.token_1_program.to_account_info(), 
            token_1_amount, 
            ctx.accounts.vault_1_mint.decimals, 
            &[&[crate::AUTH_SEED.as_bytes(),&[ctx.bumps.authority]]]
        )?;
        pool_state.recent_epoch = Clock::get()?.epoch;
        Ok(())
}