use cosmwasm_std::Uint128;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub pool_name: String,
    pub beneficiary: String,
    pub fee_collector: String,
    pub moneymarket: String,
    pub dp_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    Deposit {}, // UST -> DP (user)
    Earn {},    // x -> UST (beneficiary)
    Configure {
        beneficiary: Option<String>,
        fee_collector: Option<String>,
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
    /// Correspond response struct is [DepositAmountResponse](crate::pool_resp::DepositAmountResponse)
    DepositAmountOf { owner: String }, // -> Uint128
    /// Correspond response struct is [TotalDepositAmountResponse](crate::pool_resp::TotalDepositAmountResponse)
    TotalDepositAmount {}, // -> Uint128
    /// Correspond response struct is [ConfigResponse](crate::pool_resp::ConfigResponse)
    Config {}, // -> Config
    /// Correspond response struct is [ClaimableRewardResponse](crate::pool_resp::ClaimableRewardResponse)
    ClaimableReward {}, // -> Uint128
}
/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {
    Transfer { list: Vec<(String, Uint128)> },
    General {},
}
