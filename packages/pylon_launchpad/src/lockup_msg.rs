use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub start: u64,
    pub period: u64,
    pub cliff: u64,
    pub reward_rate: Decimal256,
    pub share_token: HumanAddr,
    pub reward_token: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // core
    Receive(Cw20ReceiveMsg),
    Update { target: Option<HumanAddr> },
    Withdraw { amount: Uint256 },
    Claim {},
    // internal
    DepositInternal { sender: HumanAddr, amount: Uint256 },
    WithdrawInternal { sender: HumanAddr, amount: Uint256 },
    ClaimInternal { sender: HumanAddr },
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
    Reward {}, // state::Reward
    BalanceOf {
        owner: HumanAddr,
    }, // -> Uint256
    ClaimableReward {
        owner: HumanAddr,
        timestamp: Option<u64>,
    }, // -> Uint256
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {
    pub temp_withdraw_start_time: u64,
    pub temp_withdraw_finish_time: u64,
}
