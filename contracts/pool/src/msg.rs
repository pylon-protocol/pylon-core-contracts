use cosmwasm_std::{HumanAddr, Uint128};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pool_name: String,
    pub beneficiary: HumanAddr, // -> convert to canonical address
    pub strategy: HumanAddr,
    pub dp_code_id: u64,
    pub stable_denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Deposit {},                 // UST -> DP (user)
    Redeem { amount: Uint128 }, // DP -> UST (user)
    ClaimReward {},             // x -> UST (beneficiary)
    RegisterDPToken {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    DepositAmountOf { owner: HumanAddr }, // -> Uint128
    TotalDepositAmount {},                // -> Uint128
    GetStrategy {},                       // -> HumanAddr (contract)
    GetBeneficiary {},                    // -> HumanAddr (contract)
    GetClaimableReward {},                // -> Uint128
}
