pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod curve;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;
pub use curve::*;
pub use utils::*;

declare_id!("5QWKxXbJjohpxLkEYk3G3yoirhRVVALZrBHzVThAnZyo");

pub mod admin {
    use anchor_lang::prelude::declare_id;
    declare_id!("GThUX1Atko4tqhN2NaiTazWSeFWMuiUvfFnyJyUghFMJ");
}

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
        // index: u16,
        trade_fee_rate: u64,
        protocol_fee_rate: u64,
        fund_fee_rate: u64,
        create_pool_fee: u64,
    ) -> Result<()> {
        assert!(trade_fee_rate < FEE_RATE_DENOMINATOR_VALUE);
        assert!(protocol_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);
        assert!(fund_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);
        assert!(fund_fee_rate + protocol_fee_rate <= FEE_RATE_DENOMINATOR_VALUE);
        instructions::process_create_amm_config(
            ctx,
            // index,
            trade_fee_rate,
            protocol_fee_rate,
            fund_fee_rate,
            create_pool_fee,
        )
    }

        /// Updates the owner of the amm config
    /// Must be called by the current owner or admin
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `trade_fee_rate`- The new trade fee rate of amm config, be set when `param` is 0
    /// * `protocol_fee_rate`- The new protocol fee rate of amm config, be set when `param` is 1
    /// * `fund_fee_rate`- The new fund fee rate of amm config, be set when `param` is 2
    /// * `new_owner`- The config's new owner, be set when `param` is 3
    /// * `new_fund_owner`- The config's new fund owner, be set when `param` is 4
    /// * `param`- The vaule can be 0 | 1 | 2 | 3 | 4, otherwise will report a error
    ///
    pub fn update_amm_config(ctx: Context<UpdateAmmConfig>, param: u8, value: u64) -> Result<()> {
        instructions::process_update_amm_config(ctx, param, value)
    }

    /// Update pool status for given vaule
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `status` - The vaule of status
    ///
    pub fn update_pool_status(ctx: Context<UpdatePoolStatus>, status: u8) -> Result<()> {
        instructions::process_update_pool_status(ctx, status)
    }

        /// Creates a pool for the given token pair and the initial price
    ///
    /// # Arguments
    ///
    /// * `ctx`- The context of accounts
    /// * `init_amount_0` - the initial amount_0 to deposit
    /// * `init_amount_1` - the initial amount_1 to deposit
    /// * `open_time` - the timestamp allowed for swap
    ///
    pub fn initialize(
        ctx: Context<Initialize>,
        init_amount_0: u64,
        init_amount_1: u64,
        open_time: u64,
    ) -> Result<()> {
        instructions::process_initialize(ctx, init_amount_0, init_amount_1, open_time)
    }

}
