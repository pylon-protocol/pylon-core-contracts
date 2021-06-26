use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub beneficiary: HumanAddr,
    pub x_denom: String,
    pub y_addr: HumanAddr,
    pub liq_x: Uint256,
    pub liq_y: Uint256, // is also a maximum cap of this pool
    pub price: Decimal256,
    pub max_cap: Uint256,
    pub total_sale_amount: Uint256,
    pub start: u64,
    pub period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Deposit {},
    Withdraw { amount: Uint256 },
    Earn {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    BalanceOf { owner: HumanAddr },
    TotalSupply {},
    CurrentPrice {},
    SimulateWithdraw { amount: Uint256 },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
