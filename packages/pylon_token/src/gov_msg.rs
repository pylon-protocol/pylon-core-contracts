use cosmwasm_std::{Binary, Decimal, Uint128};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::common::OrderBy;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub voting_token: String,
    pub quorum: Decimal,
    pub threshold: Decimal,
    pub voting_period: u64,
    pub timelock_period: u64,
    pub proposal_deposit: Uint128,
    pub snapshot_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollMsg {
    CastVote {
        poll_id: u64,
        vote: VoteOption,
        amount: Uint128,
    },
    Execute {
        poll_id: u64,
    },
    ExecuteMsgs {
        poll_id: u64,
    },
    Snapshot {
        poll_id: u64,
    },
    End {
        poll_id: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    Unstake { amount: Option<Uint128> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Receive(Cw20ReceiveMsg),
    UpdateConfig {
        owner: Option<String>,
        quorum: Option<Decimal>,
        threshold: Option<Decimal>,
        voting_period: Option<u64>,
        timelock_period: Option<u64>,
        proposal_deposit: Option<Uint128>,
        snapshot_period: Option<u64>,
    },
    Poll(PollMsg),
    Staking(StakingMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Cw20HookMsg {
    /// StakeVotingTokens a user can stake their mirror token to receive rewards
    /// or do vote on polls
    Stake {},
    /// CreatePoll need to receive deposit from a proposer
    CreatePoll {
        title: String,
        description: String,
        link: Option<String>,
        execute_msgs: Option<Vec<PollExecuteMsg>>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PollExecuteMsg {
    pub order: u64,
    pub contract: String,
    pub msg: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    APIVersion {},
    Config {},
    State {},
    Staker {
        address: String,
    },
    Stakers {
        start_after: Option<String>,
        limit: Option<u32>,
        order: Option<OrderBy>,
    },
    Poll {
        poll_id: u64,
    },
    Polls {
        filter: Option<PollStatus>,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
    Voters {
        poll_id: u64,
        start_after: Option<String>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoterInfo {
    pub vote: VoteOption,
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    InProgress,
    Passed,
    Rejected,
    Executed,
    Expired, // Deprecated
    Failed,
}

impl fmt::Display for PollStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteOption {
    Yes,
    No,
}

impl fmt::Display for VoteOption {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self == VoteOption::Yes {
            write!(f, "yes")
        } else {
            write!(f, "no")
        }
    }
}
