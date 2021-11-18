use cosmwasm_std::{Addr, BlockInfo, Decimal, StdResult, Storage, Uint128};
use cosmwasm_storage::{Bucket, ReadonlyBucket};
use pylon_token::common::OrderBy;
use pylon_utils::range::{calc_range_end, calc_range_start};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::cmp::{max, min};
use std::convert::TryInto;

use crate::constant::{DEFAULT_QUERY_LIMIT, MAX_QUERY_LIMIT};

static PREFIX_AIRDROP: &[u8] = b"airdrop";
static PREFIX_AIRDROP_REWARD: &[u8] = b"airdrop_reward";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub start: u64,
    pub period: u64,
    pub reward_token: Addr,
    pub reward_rate: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub last_update_time: u64,
    pub reward_per_token_stored: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Airdrop {
    pub config: Config,
    pub state: State,
}

impl Default for Airdrop {
    fn default() -> Self {
        Airdrop {
            config: Config {
                start: 0,
                period: 0,
                reward_token: Addr::unchecked(""),
                reward_rate: Default::default(),
            },
            state: State {
                last_update_time: 0,
                reward_per_token_stored: Default::default(),
            },
        }
    }
}

impl Airdrop {
    pub fn finish(&self) -> u64 {
        self.config.start + self.config.period
    }

    pub fn applicable_time(&self, block: &BlockInfo) -> u64 {
        min(self.finish(), max(self.config.start, block.time.seconds()))
    }

    pub fn load(storage: &dyn Storage, id: &u64) -> Option<Airdrop> {
        ReadonlyBucket::new(storage, PREFIX_AIRDROP)
            .may_load(&id.to_be_bytes())
            .unwrap()
    }

    pub fn load_range(
        storage: &dyn Storage,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<(u64, Airdrop)>> {
        let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end(start_after), OrderBy::Desc),
        };

        ReadonlyBucket::new(storage, PREFIX_AIRDROP)
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(
                |item: StdResult<(Vec<u8>, Airdrop)>| -> StdResult<(u64, Airdrop)> {
                    let (k, v) = item.unwrap();
                    Ok((u64::from_be_bytes(k.try_into().unwrap()), v))
                },
            )
            .collect()
    }

    pub fn save(storage: &mut dyn Storage, id: &u64, airdrop: &Airdrop) -> StdResult<()> {
        Bucket::new(storage, PREFIX_AIRDROP).save(&id.to_be_bytes(), airdrop)
    }
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Reward {
    pub reward: Uint128,
    pub reward_per_token_paid: Decimal,
}

impl Reward {
    pub fn load(storage: &dyn Storage, address: &Addr, id: &u64) -> StdResult<Reward> {
        Ok(
            ReadonlyBucket::multilevel(storage, &[PREFIX_AIRDROP_REWARD, address.as_bytes()])
                .may_load(&id.to_be_bytes())?
                .unwrap_or_default(),
        )
    }

    pub fn load_range(
        storage: &dyn Storage,
        address: &Addr,
        start_after: Option<u64>,
        limit: Option<u32>,
        order_by: Option<OrderBy>,
    ) -> StdResult<Vec<(u64, Reward)>> {
        let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
        let (start, end, order_by) = match order_by {
            Some(OrderBy::Asc) => (calc_range_start(start_after), None, OrderBy::Asc),
            _ => (None, calc_range_end(start_after), OrderBy::Desc),
        };

        ReadonlyBucket::multilevel(storage, &[PREFIX_AIRDROP_REWARD, address.as_bytes()])
            .range(start.as_deref(), end.as_deref(), order_by.into())
            .take(limit)
            .map(
                |item: StdResult<(Vec<u8>, Reward)>| -> StdResult<(u64, Reward)> {
                    let (k, v) = item.unwrap();
                    Ok((u64::from_be_bytes(k.try_into().unwrap()), v))
                },
            )
            .collect()
    }

    pub fn save(
        storage: &mut dyn Storage,
        address: &Addr,
        airdrop_id: &u64,
        reward: &Reward,
    ) -> StdResult<()> {
        let mut bucket: Bucket<Reward> =
            Bucket::multilevel(storage, &[PREFIX_AIRDROP_REWARD, address.as_bytes()]);
        bucket.save(&airdrop_id.to_be_bytes(), reward)
    }

    pub fn remove(storage: &mut dyn Storage, address: &Addr, airdrop_id: &u64) {
        let mut bucket: Bucket<Reward> =
            Bucket::multilevel(storage, &[PREFIX_AIRDROP_REWARD, address.as_bytes()]);
        bucket.remove(&airdrop_id.to_be_bytes())
    }
}
