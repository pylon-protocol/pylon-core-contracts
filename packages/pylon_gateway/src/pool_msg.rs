use cosmwasm_bignumber::Uint256;
use cosmwasm_std::HumanAddr;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    // we supports only linear distribution
    pub start_time: u64,
    pub sale_period: u64,
    pub sale_amount: Uint256,

    pub depositable: bool,
    pub withdrawable: bool,
    pub cliff_period: u64,
    pub vesting_period: u64,
    pub unbonding_period: u64,

    pub staking_token: HumanAddr,
    pub reward_token: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // core
    Update { target: Option<HumanAddr> },
    Receive(Cw20ReceiveMsg),
    Withdraw { amount: Uint256 },
    ClaimReward {},
    ClaimWithdrawal {},
    // internal
    DepositInternal { sender: HumanAddr, amount: Uint256 },
    WithdrawInternal { sender: HumanAddr, amount: Uint256 },
    ClaimRewardInternal { sender: HumanAddr },
    ClaimWithdrawalInternal { sender: HumanAddr },
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
        address: HumanAddr,
    }, // -> Uint256
    ClaimableReward {
        address: HumanAddr,
        timestamp: Option<u64>,
    }, // -> Uint256
    ClaimableWithdrawal {
        address: HumanAddr,
        timestamp: Option<u64>,
    },
    PendingWithdrawals {
        address: HumanAddr,
        page: Option<u32>,
        limit: Option<u32>,
    },
}

/// We currently take no arguments for migrations
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
