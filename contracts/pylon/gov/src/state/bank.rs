use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket};
use pylon_token::gov::VoterInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static PREFIX_BANK: &[u8] = b"bank";

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenManager {
    pub share: Uint128,                        // total staked balance
    pub locked_balance: Vec<(u64, VoterInfo)>, // maps poll_id to weight voted
}

pub fn bank_r(storage: &dyn Storage) -> ReadonlyBucket<TokenManager> {
    bucket_read(storage, PREFIX_BANK)
}

pub fn bank_w(storage: &mut dyn Storage) -> Bucket<TokenManager> {
    bucket(storage, PREFIX_BANK)
}
