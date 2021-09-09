use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage};
use std::cmp::max;
use std::ops::{Add, Div, Mul, Sub};

use crate::state::{config, reward, user};

pub fn calculate_reward_per_token<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    reward: &reward::Reward,
    timestamp: &u64,
) -> StdResult<Decimal256> {
    let config = config::read(&deps.storage).unwrap().distribution_config;

    let period =
        Uint256::from(max(*timestamp, reward.last_update_time).sub(reward.last_update_time));
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
    reward: &reward::Reward,
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
