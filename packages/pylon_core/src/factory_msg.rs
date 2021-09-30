use cosmwasm_bignumber::Decimal256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub pool_code_id: u64,
    pub token_code_id: u64,
    pub fee_rate: Decimal256,
    pub fee_collector: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigureMsg {
    pub owner: Option<String>,
    pub pool_code_id: Option<u64>,
    pub token_code_id: Option<u64>,
    pub fee_rate: Option<Decimal256>,
    pub fee_collector: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Configure(ConfigureMsg),
    CreatePool {
        pool_name: String,
        beneficiary: String,
        yield_adapter: String,
    },
    RegisterPool {
        pool_id: u64,
    },
    RegisterAdapter {
        address: String,
    },
    UnregisterAdapter {
        address: String,
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
        address: String,
    },
    AdapterInfos {
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
