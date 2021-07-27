use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, StdResult, Storage};
use cosmwasm_storage::{Bucket, ReadonlyBucket};

pub static PREFIX_ADAPTER: &[u8] = b"adapter";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Adapter {
    pub address: CanonicalAddr,
    pub fee_rate: Decimal256,
}

pub fn store<S: Storage>(
    storage: &mut S,
    address: CanonicalAddr,
    adapter: &Adapter,
) -> StdResult<()> {
    Bucket::new(PREFIX_ADAPTER, storage).save(address.as_slice(), adapter)
}

pub fn read<S: ReadonlyStorage>(storage: &S, address: CanonicalAddr) -> StdResult<Adapter> {
    match ReadonlyBucket::new(PREFIX_ADAPTER, storage).may_load(address.as_slice())? {
        Some(adapter) => Ok(adapter),
        None => Ok(Adapter {
            address: CanonicalAddr::default(),
            fee_rate: Decimal256::zero(),
        }),
    }
}

pub fn remove<S: Storage>(storage: &mut S, address: CanonicalAddr) {
    Bucket::<S, Adapter>::new(PREFIX_ADAPTER, storage).remove(address.as_slice())
}
