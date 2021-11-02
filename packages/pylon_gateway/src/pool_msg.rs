use cosmwasm_bignumber::Uint256;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub start: u64,
    pub period: u64,
    pub cliff: u64,
    pub share_token: String,
    pub reward_token: String,
    pub reward_amount: Uint256,
    pub cap_strategy: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConfigureMsg {
    Pool {
        owner: Option<String>,
        share_token: Option<String>,
        reward_token: Option<String>,
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
    SubReward {
        amount: Uint256,
    },
    AddReward {
        amount: Uint256,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
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
    },
    Reward {}, // state::Reward
    BalanceOf {
        owner: String,
    }, // -> Uint256
    AvailableCapOf {
        address: String,
    },
    ClaimableReward {
        owner: String,
    }, // -> Uint256
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MigrateMsg {}
