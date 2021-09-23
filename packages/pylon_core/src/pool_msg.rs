use cosmwasm_std::HumanAddr;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pool_name: String,
    pub beneficiary: HumanAddr,
    pub fee_collector: HumanAddr,
    pub moneymarket: HumanAddr,
    pub dp_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    RegisterDPToken {},
    Receive(Cw20ReceiveMsg),
    Deposit {}, // UST -> DP (user)
    Earn {},    // x -> UST (beneficiary)
    Configure {
        beneficiary: HumanAddr,
        fee_collector: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Redeem {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    DepositAmountOf { owner: HumanAddr }, // -> Uint128
    TotalDepositAmount {},                // -> Uint128
    Config {},                            // -> Config
    ClaimableReward {},                   // -> Uint128
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
