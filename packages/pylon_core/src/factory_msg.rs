use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub pool_code_id: u64,
    pub token_code_id: u64,
    pub fee_collector: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Configure {
        owner: HumanAddr,
        pool_code_id: u64,
        token_code_id: u64,
        fee_collector: HumanAddr,
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
        fee_rate: Decimal256,
    },
    UnregisterAdapter {
        address: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    State {},
    PoolInfo { pool_id: u64 },
    AdapterInfo { address: HumanAddr },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
