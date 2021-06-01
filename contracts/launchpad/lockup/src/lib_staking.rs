use std::ops::{Add, Div, Mul, Sub};

use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage};

use crate::state;

pub fn calculate_reward_per_token<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    reward: &state::Reward,
    timestamp: &u64,
) -> StdResult<Decimal256> {
    let config: state::Config = state::read_config(&deps.storage)?;

    let period = Uint256::from(timestamp.sub(reward.last_update_time));
    let total_deposit = Uint256::from(reward.total_deposit);

    Ok(Decimal256::from_uint256(period)
        .mul(config.reward_rate)
        .div(Decimal256::from_uint256(total_deposit)))
}

pub fn calculate_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    reward: &state::Reward,
    user: &state::User,
    timestamp: Option<u64>,
) -> StdResult<Uint256> {
    let mut rpt = reward
        .reward_per_token_stored
        .sub(user.reward_per_token_paid);

    if let Some(timestamp) = timestamp {
        if reward.last_update_time.gt(&timestamp) {
            return Err(StdError::generic_err(
                "Staking: timestamp must be greater than last update time",
            ));
        }

        if reward.last_update_time.ne(&timestamp) {
            rpt = rpt.add(calculate_reward_per_token(deps, reward, &timestamp)?);
        }
    }

    Ok(Uint256::from(user.reward).add(rpt * user.amount).into())
}
