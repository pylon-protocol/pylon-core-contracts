use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw20::Cw20ReceiveMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pool_name: String,
    pub beneficiary: HumanAddr, // -> convert to canonical address
    pub moneymarket: HumanAddr,
    pub dp_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive(Cw20ReceiveMsg),
    Deposit {},     // UST -> DP (user)
    ClaimReward {}, // x -> UST (beneficiary)
    RegisterDPToken {},
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
    GetBeneficiary {},                    // -> HumanAddr (contract)
    GetMoneyMarket {},                    // -> HumanAddr (contract)
    GetAToken {},                         // -> HumanAddr (contract)
    GetStableDenom {},                    // -> String
    GetClaimableReward {},                // -> Uint128
    GetDPToken {},                        // -> HumanAddr (contract)
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
