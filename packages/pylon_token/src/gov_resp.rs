use cosmwasm_std::{Decimal, Uint128, Uint256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::gov_msg::{PollExecuteMsg, PollStatus, VoteOption, VoterInfo};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct APIVersionResponse {
    pub version: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub owner: String,
    pub pylon_token: String,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub poll_count: u64,
    pub total_share: Uint128,
    pub total_deposit: Uint128,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct PollResponse {
    pub id: u64,
    pub creator: String,
    pub status: PollStatus,
    pub end_height: u64,
    pub title: String,
    pub category: String,
    pub description: String,
    pub link: Option<String>,
    pub deposit_amount: Uint128,
    pub execute_data: Option<Vec<PollExecuteMsg>>,
    pub yes_votes: Uint128, // balance
    pub no_votes: Uint128,  // balance
    pub staked_amount: Option<Uint128>,
    pub total_balance_at_end_poll: Option<Uint128>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct PollsResponse {
    pub polls: Vec<PollResponse>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct PollCountResponse {
    pub poll_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct StakerResponse {
    pub balance: Uint128,
    pub share: Uint128,
    pub claimable_airdrop: Vec<(String, Uint256)>,
    pub locked_balance: Vec<(u64, VoterInfo)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct StakersResponse {
    pub stakers: Vec<(String, StakerResponse)>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct VotersResponseItem {
    pub voter: String,
    pub vote: VoteOption,
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct VotersResponse {
    pub voters: Vec<VotersResponseItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct AirdropResponse {
    pub start: u64,
    pub period: u64,
    pub reward_token: String,
    pub reward_amount: Uint256,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
pub struct AirdropsResponse {
    pub airdrops: Vec<(u64, AirdropResponse)>,
}
