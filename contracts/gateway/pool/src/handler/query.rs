use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{to_binary, Api, Binary, Extern, HumanAddr, Querier, StdResult, Storage};
use pylon_gateway::pool_resp as resp;
use std::ops::{Mul, Sub};

use crate::querier::pool;
use crate::state::{config, state, user, withdrawal};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let config = config::read(&deps.storage)?;
    let period = config.finish_time.sub(config.start_time);

    to_binary(&resp::ConfigResponse {
        owner: deps.api.human_address(&config.owner)?,
        start_time: config.start_time.clone(),
        sale_period: period,
        sale_amount: Uint256::from(period).mul(config.reward_rate),

        depositable: config.depositable.clone(),
        withdrawable: config.withdrawable.clone(),
        clff_period: config.cliff_period.clone(),
        vesting_period: config.vesting_period.clone(),
        unbonding_period: config.unbonding_period.clone(),

        staking_token: deps.api.human_address(&config.staking_token)?,
        reward_token: deps.api.human_address(&config.reward_token)?,
    })
}

pub fn reward<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let reward = state::read(&deps.storage)?;

    to_binary(&resp::RewardResponse {
        total_deposit: reward.total_deposit,
        last_update_time: reward.last_update_time,
    })
}

pub fn balance_of<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
) -> StdResult<Binary> {
    let user = user::read(&deps.storage, &deps.api.canonical_address(&owner)?)?;

    to_binary(&resp::BalanceOfResponse {
        amount: user.amount,
    })
}

pub fn claimable_reward<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
    timestamp: Option<u64>,
) -> StdResult<Binary> {
    let reward = state::read(&deps.storage)?;
    let user = user::read(&deps.storage, &deps.api.canonical_address(&owner)?)?;

    to_binary(&resp::ClaimableRewardResponse {
        amount: pool::calculate_rewards(deps, &reward, &user, timestamp)?,
    })
}

pub fn claimable_withdrawal<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
    timestamp: Option<u64>,
) -> StdResult<Binary> {
    to_binary(&resp::ClaimableWithdrawalResponse {
        amount: pool::calculate_withdrawal_amount(
            deps,
            &deps.api.canonical_address(&owner)?,
            timestamp,
        )?,
    })
}

pub fn pending_withdrawals<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    owner: HumanAddr,
    page: Option<u32>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    let page = match page {
        Some(page) => page,
        None => 0,
    };

    to_binary(&resp::PendingWithdrawalsResponse {
        withdrawals: withdrawal::batch_read(
            deps,
            &deps.api.canonical_address(&owner)?,
            u64::from(page.mul(match limit {
                Some(limit) => limit,
                None => 0,
            })),
            limit,
        )?
        .iter()
        .map(|elem| resp::Withdrawal {
            amount: elem.amount.clone(),
            period: elem.period.clone(),
            emitted: elem.emitted.clone(),
        })
        .collect(),
    })
}
