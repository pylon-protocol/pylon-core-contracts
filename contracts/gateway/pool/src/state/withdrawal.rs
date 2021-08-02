use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{CanonicalAddr, HumanAddr, Order, ReadonlyStorage, StdResult};
use cosmwasm_storage::ReadonlyBucket;
use std::io::Read;
use std::ops::Add;

pub static PREFIX_WITHDRAWAL: &[u8] = b"withdrawal";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Withdrawal {
    pub amount: Uint256,
    pub period: u64,
    pub emitted: u64,
}

fn to_key(owner: &CanonicalAddr, index: u64) -> &[u8] {
    [owner.as_slice(), ":".as_bytes(), index.to_be_bytes()].concat()
}

pub fn read<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    index: Uint256,
) -> StdResult<Withdrawal> {
    let key = to_key(owner, index);
    match ReadonlyBucket::new(PREFIX_WITHDRAWAL, storage).may_load(key)? {
        Some(withdrawal) => Ok(withdrawal),
        None => Ok(Withdrawal {
            amount: Uint256::zero(),
            period: 0,
            emitted: 0,
        }),
    }
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn batch_read<S: ReadonlyStorage>(
    storage: &S,
    owner: &CanonicalAddr,
    start: Uint256,
    limit: Option<u32>,
) -> StdResult<Withdrawal> {
    let key = to_key(owner, start);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    ReadonlyBucket::new(PREFIX_WITHDRAWAL, storage)
        .range(Option::from(start_key), None, Order::Ascending)
        .take(limit)
        .map(|elem| {
            let (k, v) = elem?;
            let borrower: HumanAddr = deps.api.human_address(&CanonicalAddr::from(k))?;
            Ok(BorrowerInfoResponse {
                borrower,
                interest_index: v.interest_index,
                reward_index: v.reward_index,
                loan_amount: v.loan_amount,
                pending_rewards: v.pending_rewards,
            })
        })
        .collect()
}
