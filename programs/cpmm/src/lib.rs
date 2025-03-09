#![allow(unexpected_cfgs)]
pub mod constants;
pub mod curve;
pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use curve::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: "cmpp",
    project_url: "https://xxx.io",
    source_code: "https://github.com/aidyfeng/cpmm",
    contacts: "link:xxx",
    policy: "xxx",
    preferred_languages: "cn",
    auditors: "xxx"
}

declare_id!("HmjcsDRAWNMJtAfKzRuGLEUoj9rXidLJDfnJ5WMMYKz1");

/* pub mod admin {
    use anchor_lang::prelude::declare_id;
    declare_id!("GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ");
} */

#[program]
pub mod cpmm {

    use super::*;

    // The configuation of AMM protocol, include trade fee and protocol fee
    /// # Arguments
    ///
    /// * `ctx`- The accounts needed by instruction.
    /// * `index` - The index of amm config, there may be multiple config.
    /// * `trade_fee_rate` - Trade fee rate, can be changed.
    /// * `protocol_fee_rate` - The rate of protocol fee within tarde fee.
    /// * `fund_fee_rate` - The rate of fund fee within tarde fee.
    ///
    pub fn create_amm_config(
        ctx: Context<CreateAmmConfig>,
        index: u16,
        trade_fee_rate: u64,
    ) -> Result<()> {
        assert!(trade_fee_rate < FEE_RATE_DENOMINATOR_VALUE);
        instructions::process_create_amm_config(ctx, index, trade_fee_rate)
    }

    /// Updates the owner of the amm config
    /// Must be called by the current owner or admin
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `trade_fee_rate`- The new trade fee rate of amm config, be set when `param` is 0
    /// * `param`- The vaule can be 0 | 1 , otherwise will report a error
    /// * `index`- The amm config index
    ///
    pub fn update_amm_config(
        ctx: Context<UpdateAmmConfig>,
        param: u8,
        value: u64,
        index: u16,
    ) -> Result<()> {
        instructions::process_update_amm_config(ctx, param, value, index)
    }

    /// Update pool status for given vaule
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `status` - The vaule of status
    ///
    pub fn update_pool_status(
        ctx: Context<UpdatePoolStatus>,
        _index: u16,
        status: u8,
    ) -> Result<()> {
        instructions::process_update_pool_status(ctx, status)
    }

    /// Creates a pool for the given token pair and the initial price
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `_index` - amm config index
    /// * `init_amount_0` - the initial amount_0 to deposit
    /// * `init_amount_1` - the initial amount_1 to deposit
    /// * `open_time` - the timestamp allowed for swap
    ///
    pub fn initialize(
        ctx: Context<Initialize>,
        _index: u16,
        init_amount_0: u64,
        init_amount_1: u64,
        open_time: u64,
    ) -> Result<()> {
        instructions::process_initialize(ctx, init_amount_0, init_amount_1, open_time)
    }

    /// deposit liquidity token into the pool
    ///
    /// # Arguments
    /// * `ctx`- The context of accounts
    /// * `_index` - amm config index
    /// * `lp_token_amount` - the lp_token amount_0 to deposit
    /// * `maximum_token_0_amount` - Maximum token 0 amount to deposit, prevents excessive slippage
    /// * `maximum_token_1_amount` - Maximum token 1 amount to deposit, prevents excessive slippage
    ///
    pub fn deposit(
        ctx: Context<Deposit>,
        _index: u16,
        lp_token_amount: u64,
        maximum_token_0_amount: u64,
        maximum_token_1_amount: u64,
    ) -> Result<()> {
        instructions::process_deposit(
            ctx,
            lp_token_amount,
            maximum_token_0_amount,
            maximum_token_1_amount,
        )
    }

    /// Withdraw lp for token0 ande token1
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `_index` - amm config index
    /// * `lp_token_amount` - Amount of pool tokens to burn. User receives an output of token a and b based on the percentage of the pool tokens that are returned.
    /// * `minimum_token_0_amount` -  Minimum amount of token 0 to receive, prevents excessive slippage
    /// * `minimum_token_1_amount` -  Minimum amount of token 1 to receive, prevents excessive slippage
    ///
    pub fn withdraw(
        ctx: Context<Withdraw>,
        _index: u16,
        lp_token_amount: u64,
        minimum_token_0_amount: u64,
        minimum_token_1_amount: u64,
    ) -> Result<()> {
        instructions::process_withdraw(
            ctx,
            lp_token_amount,
            minimum_token_0_amount,
            minimum_token_1_amount,
        )
    }

    /// Swap the tokens in the pool base input amount
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `_index` - amm config index
    /// * `amount_in` -  input amount to transfer, output to DESTINATION is based on the exchange rate
    /// * `minimum_amount_out` -  Minimum amount of output token, prevents excessive slippage
    ///
    pub fn swap_base_input(
        ctx: Context<Swap>,
        _index: u16,
        amount_in: u64,
        minimum_amount_out: u64,
    ) -> Result<()> {
        instructions::process_swap_base_input(ctx, amount_in, minimum_amount_out)
    }

    /// Swap the tokens in the pool base output amount
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `_index` - amm config index
    /// * `amount_out` -  amount of output token
    /// * `max_amount_in` -  input amount prevents excessive slippage
    ///
    pub fn swap_base_output(
        ctx: Context<Swap>,
        _index: u16,
        amount_out: u64,
        max_amount_in: u64,
    ) -> Result<()> {
        instructions::process_swap_base_output(ctx, amount_out, max_amount_in)
    }
}
