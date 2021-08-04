use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::HumanAddr;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub id: u64,
    pub name: String,
    pub factory: HumanAddr,
    pub beneficiary: HumanAddr,
    pub fee_collector: HumanAddr,
    pub yield_adapter: HumanAddr,
    pub input_denom: String,
    pub yield_token: HumanAddr,
    pub dp_token: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DepositAmountResponse {
    pub amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TotalDepositAmountResponse {
    pub amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableRewardResponse {
    pub amount: Uint256,
    pub fee: Uint256,
}
