use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub beneficiary: HumanAddr,
    pub pool_x_denom: String,
    pub pool_y_addr: HumanAddr,
    pub pool_liq_x: Uint256,
    pub pool_liq_y: Uint256, // is also a maximum cap of this pool
    pub base_price: Decimal256,
    pub min_user_cap: Uint256,
    pub max_user_cap: Uint256,
    pub staking_contract: HumanAddr,
    pub min_stake_amount: Uint256,
    pub max_stake_amount: Uint256,
    pub additional_cap_per_token: Decimal256,
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
    AvailableCapOf { address: HumanAddr },
    TotalSupply {},
    CurrentPrice {},
    SimulateWithdraw { amount: Uint256 },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
