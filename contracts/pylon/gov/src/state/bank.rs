use crate::constant::{DEFAULT_QUERY_LIMIT, MAX_QUERY_LIMIT};
use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Uint128};
use cosmwasm_storage::{Bucket, ReadonlyBucket};
use pylon_token::common::OrderBy;
use pylon_utils::range::{calc_range_end_addr, calc_range_start_addr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::poll::VoterInfo;

static PREFIX_BANK: &[u8] = b"bank";

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenManager {
    pub share: Uint128,                        // total staked balance
    pub locked_balance: Vec<(u64, VoterInfo)>, // maps poll_id to weight voted
}

impl TokenManager {
    pub fn load(storage: &dyn Storage, address: &CanonicalAddr) -> StdResult<TokenManager> {
        Ok(ReadonlyBucket::new(storage, PREFIX_BANK)
            .may_load(address.as_slice())?
            .unwrap_or_default())
    }

    pub fn load_range(
        storage: &dyn Storage,
        start_after: Option<CanonicalAddr>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<(CanonicalAddr, TokenManager)>> {
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start_addr(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end_addr(start_after), OrderBy::Desc),
        };
        let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

        ReadonlyBucket::new(storage, PREFIX_BANK)
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(
                |elem: StdResult<(Vec<u8>, TokenManager)>| -> StdResult<(CanonicalAddr, TokenManager)> {
                    let (k, v) = elem.unwrap();
                    Ok((CanonicalAddr::from(k), v))
                },
            )
            .collect()
    }

    pub fn save(
        storage: &mut dyn Storage,
        address: &CanonicalAddr,
        manager: &TokenManager,
    ) -> StdResult<()> {
        Bucket::new(storage, PREFIX_BANK).save(address.as_slice(), manager)
    }
}
