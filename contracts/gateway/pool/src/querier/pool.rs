use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Api, CanonicalAddr, Extern, Querier, StdError, StdResult, Storage};
use std::ops::{Add, Div, Mul, Sub};

use crate::state::{config, state, user, withdrawal};

pub fn calculate_reward_per_token<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    reward: &state::State,
    timestamp: &u64,
) -> StdResult<Decimal256> {
    let config = config::read(&deps.storage)?;
    let period = Uint256::from(timestamp.sub(reward.last_update_time));
    let total_deposit = reward.total_deposit;

    if total_deposit.eq(&Uint256::zero()) {
        Ok(Decimal256::zero())
    } else {
        Ok(Decimal256::from_uint256(period)
            .mul(config.reward_rate)
            .div(Decimal256::from_uint256(total_deposit)))
    }
}

pub fn calculate_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    reward: &state::State,
    user: &user::User,
    timestamp: Option<u64>,
) -> StdResult<Uint256> {
    let mut rpt = reward
        .reward_per_token_stored
        .sub(user.reward_per_token_paid);

    if let Some(timestamp) = timestamp {
        if reward.last_update_time.gt(&timestamp) {
            return Err(StdError::generic_err(
                "Lockup: timestamp must be greater than last update time",
            ));
        }

        if reward.last_update_time.ne(&timestamp) {
            rpt = rpt.add(calculate_reward_per_token(deps, reward, &timestamp)?);
        }
    }

    Ok(user.reward.add(rpt * user.amount))
}

pub fn calculate_withdrawal_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: &CanonicalAddr,
    blocktime: Option<u64>,
) -> StdResult<Uint256> {
    let user = user::read(&deps.storage, owner)?;
    let index = match blocktime {
        Some(blocktime) => fetch_claimable_withdrawal_index(deps, owner, blocktime)?,
        None => user.next_withdrawal_index.sub(1),
    };

    let from = withdrawal::read(&deps.storage, owner, user.claimed_withdrawal_index)?;
    let to = withdrawal::read(&deps.storage, owner, index)?;

    Ok(to.accumulated.sub(from.accumulated))
}

pub fn fetch_claimable_withdrawal_index<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: &CanonicalAddr,
    _blocktime: u64,
) -> StdResult<u64> {
    // TODO: binary search
    let user = user::read(&deps.storage, owner)?;
    let _middle = user
        .next_withdrawal_index
        .sub(user.claimed_withdrawal_index)
        .div(2)
        .add(user.claimed_withdrawal_index);

    Ok(user.next_withdrawal_index.sub(1))

    // loop {
    //     let withdrawal = withdrawal::read(&deps.storage, owner, middle)?;
    //     if withdrawal.is_claimable(&blocktime) {}
    // }
    // Ok(0)
}
