use cosmwasm_std::{Binary, CanonicalAddr, StdError, StdResult, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use pylon_token::gov::{PollStatus, VoterInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

const MIN_TITLE_LENGTH: usize = 4;
const MAX_TITLE_LENGTH: usize = 64;
const MIN_DESC_LENGTH: usize = 4;
const MAX_DESC_LENGTH: usize = 1024;
const MIN_LINK_LENGTH: usize = 12;
const MAX_LINK_LENGTH: usize = 128;

static KEY_TMP_POLL_ID: &[u8] = b"tmp_poll_id";

static PREFIX_POLL_INDEXER: &[u8] = b"poll_indexer";
static PREFIX_POLL_VOTER: &[u8] = b"poll_voter";
static PREFIX_POLL: &[u8] = b"poll";

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct ExecuteData {
    pub order: u64,
    pub contract: CanonicalAddr,
    pub msg: Binary,
}
impl Eq for ExecuteData {}

impl Ord for ExecuteData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.order.cmp(&other.order)
    }
}

impl PartialOrd for ExecuteData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ExecuteData {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Poll {
    pub id: u64,
    pub creator: CanonicalAddr,
    pub status: PollStatus,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub end_height: u64,
    pub title: String,
    pub description: String,
    pub link: Option<String>,
    pub execute_data: Option<Vec<ExecuteData>>,
    pub deposit_amount: Uint128,
    /// Total balance at the end poll
    pub total_balance_at_end_poll: Option<Uint128>,
    pub staked_amount: Option<Uint128>,
}

pub fn poll_r(storage: &dyn Storage) -> ReadonlyBucket<Poll> {
    bucket_read(storage, PREFIX_POLL)
}

pub fn poll_w(storage: &mut dyn Storage) -> Bucket<Poll> {
    bucket(storage, PREFIX_POLL)
}

// temp poll
pub fn tmp_poll_id_r(storage: &dyn Storage) -> ReadonlySingleton<u64> {
    singleton_read(storage, KEY_TMP_POLL_ID)
}

pub fn tmp_poll_id_w(storage: &mut dyn Storage) -> Singleton<u64> {
    singleton(storage, KEY_TMP_POLL_ID)
}

// indexer
pub fn poll_indexer_r<'a>(
    storage: &'a dyn Storage,
    status: &PollStatus,
) -> ReadonlyBucket<'a, bool> {
    ReadonlyBucket::multilevel(
        storage,
        &[PREFIX_POLL_INDEXER, status.to_string().as_bytes()],
    )
}

pub fn poll_indexer_w<'a>(storage: &'a mut dyn Storage, status: &PollStatus) -> Bucket<'a, bool> {
    Bucket::multilevel(
        storage,
        &[PREFIX_POLL_INDEXER, status.to_string().as_bytes()],
    )
}

// voter
pub fn poll_voter_r(storage: &dyn Storage, poll_id: u64) -> ReadonlyBucket<VoterInfo> {
    ReadonlyBucket::multilevel(storage, &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()])
}

pub fn poll_voter_w(storage: &mut dyn Storage, poll_id: u64) -> Bucket<VoterInfo> {
    Bucket::multilevel(storage, &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()])
}

impl Poll {
    pub fn validate(&self) -> StdResult<()> {
        Poll::validate_title(self.title.as_str())?;
        Poll::validate_description(self.description.as_str())?;
        Poll::validate_link(&self.link)?;
        Ok(())
    }

    /// validate_title returns an error if the title is invalid
    pub fn validate_title(title: &str) -> StdResult<()> {
        if title.len() < MIN_TITLE_LENGTH {
            Err(StdError::generic_err("Title too short"))
        } else if title.len() > MAX_TITLE_LENGTH {
            Err(StdError::generic_err("Title too long"))
        } else {
            Ok(())
        }
    }

    /// validate_description returns an error if the description is invalid
    pub fn validate_description(description: &str) -> StdResult<()> {
        if description.len() < MIN_DESC_LENGTH {
            Err(StdError::generic_err("Description too short"))
        } else if description.len() > MAX_DESC_LENGTH {
            Err(StdError::generic_err("Description too long"))
        } else {
            Ok(())
        }
    }

    /// validate_link returns an error if the link is invalid
    pub fn validate_link(link: &Option<String>) -> StdResult<()> {
        if let Some(link) = link {
            if link.len() < MIN_LINK_LENGTH {
                Err(StdError::generic_err("Link too short"))
            } else if link.len() > MAX_LINK_LENGTH {
                Err(StdError::generic_err("Link too long"))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}
