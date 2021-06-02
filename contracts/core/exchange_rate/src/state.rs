use cosmwasm_std::{CanonicalAddr, HumanAddr, ReadonlyStorage, StdResult, Storage};
use cosmwasm_storage::{Bucket, ReadonlyBucket, ReadonlySingleton, Singleton};

use cosmwasm_bignumber::Decimal256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static KEY_CONFIG: &[u8] = b"config";
pub static PREFIX_TOKEN: &[u8] = b"token";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: HumanAddr,
}

pub fn store_config<S: Storage>(storage: &mut S, data: &Config) -> StdResult<()> {
    Singleton::new(storage, KEY_CONFIG).save(data)
}

pub fn read_config<S: Storage>(storage: &S) -> StdResult<Config> {
    ReadonlySingleton::new(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Status {
    NEUTRAL,
    RUNNING,
    STOPPED,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    pub status: Status,
    pub exchange_rate: Decimal256,
    pub epoch_period: u64,
    pub weight: Decimal256,
    pub last_updated_at: u64,
}

pub fn read_token<S: ReadonlyStorage>(storage: &S, token: &CanonicalAddr) -> StdResult<Token> {
    match ReadonlyBucket::new(PREFIX_TOKEN, storage).may_load(token.as_slice())? {
        Some(token) => Ok(token),
        None => Ok(Token {
            status: Status::NEUTRAL,
            exchange_rate: Decimal256::zero(),
            epoch_period: 0,
            weight: Decimal256::zero(),
            last_updated_at: 0,
        }),
    }
}

pub fn store_token<S: Storage>(
    storage: &mut S,
    token_addr: &CanonicalAddr,
    token: &Token,
) -> StdResult<()> {
    Bucket::new(PREFIX_TOKEN, storage).save(token_addr.as_slice(), token)
}

pub fn remove_token<S: Storage>(storage: &mut S, token_addr: &CanonicalAddr) {
    Bucket::<S, Token>::new(PREFIX_TOKEN, storage).remove(token_addr.as_slice())
}
