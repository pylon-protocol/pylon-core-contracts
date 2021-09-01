use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: HumanAddr,
    pub pool_code_id: u64,
    pub token_code_id: u64,
    pub fee_rate: Decimal256,
    pub fee_collector: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfoResponse {
    pub id: u64,
    pub address: HumanAddr,
    pub dp_address: HumanAddr,
    pub yield_adapter: HumanAddr,
    pub accumulated_fee: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdapterInfoResponse {
    pub address: HumanAddr,
    pub input_denom: String,
    pub yield_token: HumanAddr,
}
