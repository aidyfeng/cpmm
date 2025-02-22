use anchor_lang::prelude::*;

pub const AMM_CONFIG_SEED: &str = "amm_config";

/// Holds the current owner of the factory
#[account]
#[derive(InitSpace)]
pub struct AmmConfig {
    /// Bump to identify PDA
    pub bump: u8,
    /// Config index
    pub index: u16,
    /// Status to control if new pool can be create
    pub disable_create_pool: bool,
    /// The trade fee, denominated in hundredths of a bip (10^-6)
    pub trade_fee_rate: u64,
}
