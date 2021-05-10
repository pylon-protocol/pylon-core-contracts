use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw20::Cw20ReceiveMsg;

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
    Withdraw {},
    Exit {},
    Claim {},
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
    BalanceOf { owner: HumanAddr },       // -> Uint128
    TotalSupply {},                       // -> Uint128
    StartTime {},                         // -> u64
    FinishTime {},                        // -> u64
    RewardRate {},                        // -> Uint256
    DPToken {},                           // -> HumanAddr (contract)
    RewardToken {},                       // -> HumanAddr (contract)
    ClaimableReward { owner: HumanAddr }, // -> Uint128
}
