use cosmwasm_std::{Binary, CanonicalAddr, StdError, StdResult, Storage, Uint128};
use cosmwasm_storage::{Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};
use pylon_token::common::OrderBy;
use pylon_token::gov_msg::{
    PollCategory as GovPollCategory, PollStatus as GovPollStatus, VoteOption as GovVoteOption,
    VoterInfo as GovVoterInfo,
};
use pylon_utils::range::{
    calc_range_end, calc_range_end_addr, calc_range_start, calc_range_start_addr,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

use crate::constant::{DEFAULT_QUERY_LIMIT, MAX_QUERY_LIMIT};

const MIN_TITLE_LENGTH: usize = 4;
const MAX_TITLE_LENGTH: usize = 64;
const MIN_CATEGORY_LENGTH: usize = 4;
const MAX_CATEGORY_LENGTH: usize = 64;
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
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    InProgress,
    Passed,
    Rejected,
    Executed,
    Failed,
}

impl From<PollStatus> for GovPollStatus {
    fn from(status: PollStatus) -> Self {
        match status {
            PollStatus::InProgress => GovPollStatus::InProgress,
            PollStatus::Passed => GovPollStatus::Passed,
            PollStatus::Rejected => GovPollStatus::Rejected,
            PollStatus::Executed => GovPollStatus::Executed,
            PollStatus::Failed => GovPollStatus::Failed,
        }
    }
}

impl From<GovPollStatus> for PollStatus {
    fn from(status: GovPollStatus) -> Self {
        match status {
            GovPollStatus::InProgress => PollStatus::InProgress,
            GovPollStatus::Passed => PollStatus::Passed,
            GovPollStatus::Rejected => PollStatus::Rejected,
            GovPollStatus::Executed => PollStatus::Executed,
            GovPollStatus::Expired => PollStatus::Rejected,
            GovPollStatus::Failed => PollStatus::Failed,
        }
    }
}

impl fmt::Display for PollStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollCategory {
    Core,
    Gateway,
    None,
}

impl From<PollCategory> for GovPollCategory {
    fn from(category: PollCategory) -> Self {
        match category {
            PollCategory::Core {} => GovPollCategory::Core,
            PollCategory::Gateway {} => GovPollCategory::Gateway,
            PollCategory::None {} => GovPollCategory::None,
        }
    }
}

impl From<GovPollCategory> for PollCategory {
    fn from(category: GovPollCategory) -> Self {
        match category {
            GovPollCategory::Core {} => PollCategory::Core,
            GovPollCategory::Gateway {} => PollCategory::Gateway,
            GovPollCategory::None {} => PollCategory::None,
        }
    }
}

impl fmt::Display for PollCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
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
    pub category: PollCategory,
    pub description: String,
    pub link: Option<String>,
    pub execute_data: Option<Vec<ExecuteData>>,
    pub deposit_amount: Uint128,
    /// Total balance at the end poll
    pub total_balance_at_end_poll: Option<Uint128>,
    pub staked_amount: Option<Uint128>,
}

impl Poll {
    pub fn may_load(storage: &dyn Storage, id: &u64) -> StdResult<Option<Poll>> {
        ReadonlyBucket::new(storage, PREFIX_POLL).may_load(&id.to_be_bytes())
    }

    pub fn load(storage: &dyn Storage, id: &u64) -> StdResult<Poll> {
        ReadonlyBucket::new(storage, PREFIX_POLL).load(&id.to_be_bytes())
    }

    pub fn load_range(
        storage: &dyn Storage,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<Poll>> {
        let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end(start_after), OrderBy::Desc),
        };

        ReadonlyBucket::new(storage, PREFIX_POLL)
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(|item| -> StdResult<Poll> {
                let (_, v) = item?;
                Ok(v)
            })
            .collect()
    }

    pub fn load_range_with_status_filter(
        storage: &dyn Storage,
        status_filter: PollStatus,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<Poll>> {
        let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end(start_after), OrderBy::Desc),
        };

        Poll::indexed_by_status_r(storage, &status_filter)
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(|item| -> StdResult<Poll> {
                let (k, _) = item?;
                ReadonlyBucket::new(storage, PREFIX_POLL).load(&k)
            })
            .collect()
    }

    pub fn load_range_with_category_filter(
        storage: &dyn Storage,
        category_filter: PollCategory,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<Poll>> {
        let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end(start_after), OrderBy::Desc),
        };

        Poll::indexed_by_category_r(storage, &category_filter)
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(|item| -> StdResult<Poll> {
                let (k, _) = item?;
                ReadonlyBucket::new(storage, PREFIX_POLL).load(&k)
            })
            .collect()
    }

    pub fn load_temp_id(storage: &dyn Storage) -> StdResult<u64> {
        ReadonlySingleton::new(storage, KEY_TMP_POLL_ID).load()
    }

    pub fn save(storage: &mut dyn Storage, id: &u64, poll: &Poll) -> StdResult<()> {
        Bucket::new(storage, PREFIX_POLL).save(&id.to_be_bytes(), poll)
    }

    pub fn save_temp_id(storage: &mut dyn Storage, id: &u64) -> StdResult<()> {
        Singleton::new(storage, KEY_TMP_POLL_ID).save(id)
    }

    pub fn index_status(storage: &mut dyn Storage, id: &u64, status: &PollStatus) -> StdResult<()> {
        Poll::indexed_by_status_w(storage, status).save(&id.to_be_bytes(), &true)
    }

    pub fn deindex_status(storage: &mut dyn Storage, id: &u64, status: &PollStatus) {
        Poll::indexed_by_status_w(storage, status).remove(&id.to_be_bytes())
    }

    pub fn index_category(
        storage: &mut dyn Storage,
        id: &u64,
        category: &PollCategory,
    ) -> StdResult<()> {
        Poll::indexed_by_category_w(storage, category).save(&id.to_be_bytes(), &true)
    }

    #[allow(dead_code)]
    pub fn deindex_category(storage: &mut dyn Storage, id: &u64, category: &PollCategory) {
        Poll::indexed_by_category_w(storage, category).remove(&id.to_be_bytes())
    }

    /* ================= INDEXES ================= */

    fn indexed_by_category_r<'a>(
        storage: &'a dyn Storage,
        category: &PollCategory,
    ) -> ReadonlyBucket<'a, bool> {
        ReadonlyBucket::multilevel(
            storage,
            &[
                PREFIX_POLL_INDEXER,
                b"category",
                category.to_string().as_bytes(),
            ],
        )
    }

    fn indexed_by_category_w<'a>(
        storage: &'a mut dyn Storage,
        category: &PollCategory,
    ) -> Bucket<'a, bool> {
        Bucket::multilevel(
            storage,
            &[
                PREFIX_POLL_INDEXER,
                b"category",
                category.to_string().as_bytes(),
            ],
        )
    }

    fn indexed_by_status_r<'a>(
        storage: &'a dyn Storage,
        status: &PollStatus,
    ) -> ReadonlyBucket<'a, bool> {
        ReadonlyBucket::multilevel(
            storage,
            &[
                PREFIX_POLL_INDEXER,
                b"status",
                status.to_string().as_bytes(),
            ],
        )
    }

    fn indexed_by_status_w<'a>(
        storage: &'a mut dyn Storage,
        status: &PollStatus,
    ) -> Bucket<'a, bool> {
        Bucket::multilevel(
            storage,
            &[
                PREFIX_POLL_INDEXER,
                "status".as_bytes(),
                status.to_string().as_bytes(),
            ],
        )
    }

    /* ================= VALIDATOR ================= */

    pub fn validate(&self) -> StdResult<()> {
        self.validate_title()?;
        self.validate_category()?;
        self.validate_description()?;
        self.validate_link()?;
        Ok(())
    }

    /// validate_title returns an error if the title is invalid
    fn validate_title(&self) -> StdResult<()> {
        if self.title.as_str().len() < MIN_TITLE_LENGTH {
            Err(StdError::generic_err("Title too short"))
        } else if self.title.as_str().len() > MAX_TITLE_LENGTH {
            Err(StdError::generic_err("Title too long"))
        } else {
            Ok(())
        }
    }

    /// validate_category returns an error if the category is invalid
    fn validate_category(&self) -> StdResult<()> {
        if self.category.to_string().len() < MIN_CATEGORY_LENGTH {
            Err(StdError::generic_err("Category too short"))
        } else if self.category.to_string().len() > MAX_CATEGORY_LENGTH {
            Err(StdError::generic_err("Category too long"))
        } else {
            Ok(())
        }
    }

    /// validate_description returns an error if the description is invalid
    fn validate_description(&self) -> StdResult<()> {
        if self.description.as_str().len() < MIN_DESC_LENGTH {
            Err(StdError::generic_err("Description too short"))
        } else if self.description.as_str().len() > MAX_DESC_LENGTH {
            Err(StdError::generic_err("Description too long"))
        } else {
            Ok(())
        }
    }

    /// validate_link returns an error if the link is invalid
    fn validate_link(&self) -> StdResult<()> {
        if let Some(link) = &self.link {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteOption {
    Yes,
    No,
}

impl From<VoteOption> for GovVoteOption {
    fn from(option: VoteOption) -> Self {
        match option {
            VoteOption::Yes => GovVoteOption::Yes,
            VoteOption::No => GovVoteOption::No,
        }
    }
}

impl From<GovVoteOption> for VoteOption {
    fn from(option: GovVoteOption) -> Self {
        match option {
            GovVoteOption::Yes {} => VoteOption::Yes,
            GovVoteOption::No {} => VoteOption::No,
        }
    }
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VoterInfo {
    pub vote: VoteOption,
    pub balance: Uint128,
}

impl From<VoterInfo> for GovVoterInfo {
    fn from(info: VoterInfo) -> Self {
        GovVoterInfo {
            vote: info.vote.into(),
            balance: info.balance,
        }
    }
}

impl From<GovVoterInfo> for VoterInfo {
    fn from(info: GovVoterInfo) -> Self {
        VoterInfo {
            vote: info.vote.into(),
            balance: info.balance,
        }
    }
}

impl VoterInfo {
    pub fn load(
        storage: &dyn Storage,
        poll_id: &u64,
        address: &CanonicalAddr,
    ) -> StdResult<VoterInfo> {
        ReadonlyBucket::multilevel(storage, &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()])
            .load(address.as_slice())
    }

    pub fn load_range(
        storage: &dyn Storage,
        poll_id: u64,
        start_after: Option<CanonicalAddr>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<(CanonicalAddr, VoterInfo)>> {
        let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start_addr(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end_addr(start_after), OrderBy::Desc),
        };

        ReadonlyBucket::multilevel(storage, &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()])
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(|item| {
                let (k, v) = item?;
                Ok((CanonicalAddr::from(k), v))
            })
            .collect()
    }

    pub fn save(
        storage: &mut dyn Storage,
        poll_id: &u64,
        address: &CanonicalAddr,
        voter_info: &VoterInfo,
    ) -> StdResult<()> {
        let mut bucket: Bucket<VoterInfo> =
            Bucket::multilevel(storage, &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()]);
        bucket.save(address.as_slice(), voter_info)
    }

    pub fn remove(storage: &mut dyn Storage, poll_id: &u64, address: &CanonicalAddr) {
        let mut bucket: Bucket<VoterInfo> =
            Bucket::multilevel(storage, &[PREFIX_POLL_VOTER, &poll_id.to_be_bytes()]);
        bucket.remove(address.as_slice())
    }
}
