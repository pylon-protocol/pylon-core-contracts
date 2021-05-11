use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{HumanAddr, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub dp_token: HumanAddr,
    pub reward_token: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive(Cw20ReceiveMsg),
    Update {},
    Withdraw { amount: Uint128 },
    Claim {},
    Exit {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    Deposit {},
    Configure {
        start_time: u64,
        period: u64,
        open_deposit: bool,
        open_withdraw: bool,
        open_claim: bool,
        reward_rate: Decimal256,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {}, // state::Config
    Reward {}, // state::Reward
    BalanceOf {
        owner: HumanAddr,
    }, // -> Uint128
    ClaimableReward {
        owner: HumanAddr,
        timestamp: Option<u64>,
    }, // -> Uint128
}
