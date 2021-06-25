use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{CanonicalAddr, HumanAddr, ReadonlyStorage, StdResult, Storage};
use cosmwasm_storage::{Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static KEY_CONFIG: &[u8] = b"config";
pub static KEY_VPOOL: &[u8] = b"vpool";
pub static PREFIX_USER: &[u8] = b"user";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub this: HumanAddr,
    pub owner: HumanAddr,
    pub beneficiary: HumanAddr,
    pub start: u64,
    pub finish: u64,
    pub price: Decimal256,
    pub total_sale_amount: Uint256,
}

pub fn store_config<S: Storage>(storage: &mut S, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config<S: Storage>(storage: &S) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VirtualPool {
    pub x_denom: String,
    pub y_addr: CanonicalAddr,
    pub liq_x: Uint256,
    pub liq_y: Uint256,
}

pub fn store_vpool<S: Storage>(storage: &mut S, data: &VirtualPool) -> StdResult<()> {
    Singleton::new(storage, KEY_VPOOL).save(data)
}

pub fn read_vpool<S: Storage>(storage: &S) -> StdResult<VirtualPool> {
    ReadonlySingleton::new(storage, KEY_VPOOL).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct User {
    pub amount: Uint256,
}

pub fn read_user<S: ReadonlyStorage>(storage: &S, owner: &CanonicalAddr) -> StdResult<User> {
    match ReadonlyBucket::new(PREFIX_USER, storage).may_load(owner.as_slice())? {
        Some(user) => Ok(user),
        None => Ok(User {
            amount: Uint256::zero(),
        }),
    }
}

pub fn store_user<S: Storage>(
    storage: &mut S,
    owner: &CanonicalAddr,
    user: &User,
) -> StdResult<()> {
    Bucket::new(PREFIX_USER, storage).save(owner.as_slice(), user)
}

pub fn remove_user<S: Storage>(storage: &mut S, owner: &CanonicalAddr) {
    Bucket::<S, User>::new(PREFIX_USER, storage).remove(owner.as_slice())
}
