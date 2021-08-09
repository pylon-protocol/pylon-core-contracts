use cosmwasm_bignumber::Uint256;
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Withdrawal {
    pub amount: Uint256,
    pub period: u64,
    pub emitted: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: HumanAddr,
    pub start_time: u64,
    pub sale_period: u64,
    pub sale_amount: Uint256,

    pub depositable: bool,
    pub withdrawable: bool,
    pub clff_period: u64,
    pub vesting_period: u64,
    pub unbonding_period: u64,

    pub staking_token: HumanAddr,
    pub reward_token: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RewardResponse {
    pub total_deposit: Uint256,
    pub last_update_time: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BalanceOfResponse {
    pub amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableRewardResponse {
    pub amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ClaimableWithdrawalResponse {
    pub amount: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PendingWithdrawalsResponse {
    pub withdrawals: Vec<Withdrawal>,
}
