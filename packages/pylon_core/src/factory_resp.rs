use cosmwasm_bignumber::{Decimal256, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub pool_code_id: u64,
    pub token_code_id: u64,
    pub fee_rate: Decimal256,
    pub fee_collector: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfoResponse {
    pub id: u64,
    pub address: String,
    pub dp_address: String,
    pub dp_total_supply: Uint256,
    pub yield_adapter: String,
    pub yield_token: String,
    pub yield_token_balance: Uint256,
    pub accumulated_reward: Uint256,
    pub accumulated_fee: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfosResponse {
    pub pool_infos: Vec<PoolInfoResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdapterInfoResponse {
    pub address: String,
    pub input_denom: String,
    pub yield_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AdapterInfosResponse {
    pub adapter_infos: Vec<AdapterInfoResponse>,
}
