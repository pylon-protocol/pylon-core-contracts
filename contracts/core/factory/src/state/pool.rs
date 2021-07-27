use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, StdResult, Storage};
use cosmwasm_storage::{Bucket, ReadonlyBucket};

pub static PREFIX_POOL: &[u8] = b"pool";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Neutral,
    Ready,
    Deployed,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Pool {
    pub id: Uint256,
    pub status: Status,
    pub address: CanonicalAddr,
}

pub fn store<S: Storage>(storage: &mut S, id: Uint256, pool: &Pool) -> StdResult<()> {
    Bucket::new(PREFIX_POOL, storage).save(id.to_string().as_bytes(), pool)
}

pub fn read<S: ReadonlyStorage>(storage: &S, id: Uint256) -> StdResult<Pool> {
    match ReadonlyBucket::new(PREFIX_POOL, storage).may_load(id.to_string().as_bytes())? {
        Some(pool) => Ok(pool),
        None => Ok(Pool {
            id: Uint256::zero(),
            status: Status::Neutral,
            address: CanonicalAddr::default(),
        }),
    }
}

pub fn remove<S: Storage>(storage: &mut S, id: Uint256) {
    Bucket::<S, Pool>::new(PREFIX_POOL, storage).remove(id.to_string().as_bytes())
}
