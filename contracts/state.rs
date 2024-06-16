use cosmwasm_std::{Addr, Uint128};
use cosmwasm_storage::{singleton, singleton_read, Singleton, ReadonlySingleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";
pub static STAKEHOLDERS_KEY: &[u8] = b"stakeholders";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub avg_block_time: u8,
    pub decimals: u8,
    pub tax: u8,
    pub rental_limit_months: u8,
    pub rental_limit_blocks: u64,
    pub total_supply: Uint128,
    pub total_supply2: Uint128,
    pub rent_per_30_day: Uint128,
    pub accumulated: Uint128,
    pub blocks_per_30_day: u64,
    pub rental_begin: u64,
    pub occupied_until: u64,
    pub name: String,
    pub symbol: String,
    pub gov: Addr,
    pub main_property_owner: Addr,
    pub tenant: Addr,
    pub revenues: Vec<(Addr, Uint128)>,
    pub shares: Vec<(Addr, Uint128)>,
    pub allowed: Vec<((Addr, Addr), Uint128)>,
    pub rent_paid_until: Vec<(Addr, u64)>,
    pub shares_offered: Vec<(Addr, Uint128)>,
    pub share_sell_price: Vec<(Addr, Uint128)>,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG_KEY)
}
