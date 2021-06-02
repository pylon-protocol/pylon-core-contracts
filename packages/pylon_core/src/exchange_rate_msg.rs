use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum HandleMsg {
    Update {
        token: HumanAddr,
    },
    ConfigToken {
        token: HumanAddr,
        exchange_rate: Option<Decimal256>,
        epoch_period: Option<u64>,
        weight: Option<Decimal256>,
    },
    AddToken {
        token: HumanAddr,
        base_rate: Decimal256,
        period: u64,
        weight: Decimal256,
    },
    Start {
        tokens: Vec<HumanAddr>,
    },
    Stop {
        tokens: Vec<HumanAddr>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum QueryMsg {
    ExchangeRateOf {
        token: HumanAddr,
        blocktime: Option<u64>,
    },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
