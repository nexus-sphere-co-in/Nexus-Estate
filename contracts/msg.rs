use cosmwasm_std::{Uint128, Addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub property_id: String,
    pub property_symbol: String,
    pub main_property_owner: Addr,
    pub tax: u8,
    pub avg_block_time: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExecuteMsg {
    pub set_tax: Option<u8>,
    pub set_avg_block_time: Option<u8>,
    pub add_stakeholder: Option<Addr>,
    pub ban_stakeholder: Option<Addr>,
    pub distribute: Option<()>,
    pub seizure_from: Option<(Addr, Addr, Uint128)>,
    pub can_pay_rent: Option<Addr>,
    pub limit_advanced_rent: Option<u8>,
    pub set_rent_per_30_day: Option<Uint128>,
    pub offer_shares: Option<(Uint128, Uint128)>,
    pub buy_shares: Option<(Addr, Uint128)>,
    pub transfer: Option<(Addr, Uint128)>,
    pub claim_ownership: Option<()>,
    pub withdraw: Option<()>,
    pub pay_rent: Option<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryMsg {
    pub show_shares_of: Addr,
    pub is_stakeholder: Addr,
    pub current_tenant_check: Addr,
}
