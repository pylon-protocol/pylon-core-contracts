use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pool_code_id: u64,
    pub token_code_id: u64,
    pub fee_rate: Decimal256,
    pub fee_collector: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Configure {
        owner: Option<HumanAddr>,
        pool_code_id: Option<u64>,
        token_code_id: Option<u64>,
        fee_rate: Option<Decimal256>,
        fee_collector: Option<HumanAddr>,
    },
    CreatePool {
        pool_name: String,
        beneficiary: HumanAddr,
        yield_adapter: HumanAddr,
    },
    RegisterPool {
        pool_id: u64,
    },
    RegisterAdapter {
        address: HumanAddr,
    },
    UnregisterAdapter {
        address: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    PoolInfo {
        pool_id: u64,
    },
    PoolInfos {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
    AdapterInfo {
        address: HumanAddr,
    },
    AdapterInfos {
        start_after: Option<HumanAddr>,
        limit: Option<u32>,
    },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
