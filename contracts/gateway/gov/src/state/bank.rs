use cosmwasm_std::{ReadonlyStorage, Storage, Uint128};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use pylon_gateway::gov::VoterInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static PREFIX_BANK: &[u8] = b"bank";

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenManager {
    pub share: Uint128,                        // total staked balance
    pub locked_balance: Vec<(u64, VoterInfo)>, // maps poll_id to weight voted
}

pub fn store<S: Storage>(storage: &mut S) -> Bucket<S, TokenManager> {
    bucket(PREFIX_BANK, storage)
}

pub fn read<S: ReadonlyStorage>(storage: &S) -> ReadonlyBucket<S, TokenManager> {
    bucket_read(PREFIX_BANK, storage)
}
