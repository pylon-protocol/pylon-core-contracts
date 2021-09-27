use cosmwasm_bignumber::{Decimal256, Uint256};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub start: u64,
    pub period: u64,
    pub cliff: u64,
    pub reward_rate: Decimal256,
    pub share_token: String,
    pub reward_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DistributionMsg {
    SubReward { amount: Uint256 },
    AddReward { amount: Uint256 },
    ShortenPeriod { time: u64 },
    LengthenPeriod { time: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigureMsg {
    Owner {
        address: String,
    },
    Deposit {
        start: Option<u64>,
        finish: Option<u64>,
        user_cap: Option<Uint256>,
        total_cap: Option<Uint256>,
    },
    Withdraw {
        strategy: Vec<(u64, u64, bool)>,
    },
    Claim {
        start: Option<u64>,
        finish: Option<u64>,
    },
    Distribution(DistributionMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // core
    Receive(Cw20ReceiveMsg),
    Update { target: Option<String> },
    Withdraw { amount: Uint256 },
    Claim {},
    // internal
    DepositInternal { sender: String, amount: Uint256 },
    WithdrawInternal { sender: String, amount: Uint256 },
    ClaimInternal { sender: String },
    // owner
    Configure(ConfigureMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {}, // state::Config
    Stakers {
        start_after: Option<String>,
        limit: Option<u32>,
        timestamp: Option<u64>,
    },
    Reward {}, // state::Reward
    BalanceOf {
        owner: String,
    }, // -> Uint256
    ClaimableReward {
        owner: String,
        timestamp: Option<u64>,
    }, // -> Uint256
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {
    V1 {},
    V1Temp {},
}
