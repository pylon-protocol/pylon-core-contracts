use cosmwasm_bignumber::{Decimal256, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub beneficiary: String,
    pub pool_x_denom: String,
    pub pool_y_addr: String,
    pub pool_liq_x: Uint256,
    pub pool_liq_y: Uint256, // is also a maximum cap of this pool
    pub base_price: Decimal256,
    pub min_user_cap: Uint256,
    pub max_user_cap: Uint256,
    pub staking_contract: String,
    pub min_stake_amount: Uint256,
    pub max_stake_amount: Uint256,
    pub additional_cap_per_token: Decimal256,
    pub total_sale_amount: Uint256,
    pub start: u64,
    pub period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Configure {
        total_sale_amount: Uint256,
        min_user_cap: Uint256,
        max_user_cap: Uint256,
    },
    Deposit {},
    Withdraw {
        amount: Uint256,
    },
    Earn {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    BalanceOf { owner: String },
    AvailableCapOf { address: String },
    TotalSupply {},
    CurrentPrice {},
    SimulateWithdraw { amount: Uint256 },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum MigrateMsg {
    Refund {},
    General {},
}
