use cosmwasm_std::*;
use pylon_core::pool_v2_resp as resp;
use pylon_utils::token;

use crate::querier::pool;
use crate::state::config;

pub fn config(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage)?;

    to_binary(&resp::ConfigResponse {
        id: config.id,
        name: config.name,
        factory: config.factory,
        beneficiary: config.beneficiary,
        yield_adapter: config.yield_adapter,
        input_denom: config.input_denom,
        yield_token: config.yield_token,
        dp_token: config.dp_token,
    })
}

pub fn deposit_amount(deps: Deps, _env: Env, owner: String) -> StdResult<Binary> {
    let config = config::read(deps.storage)?;

    to_binary(&resp::DepositAmountResponse {
        amount: token::balance_of(deps, config.dp_token, owner)?,
    })
}

pub fn total_deposit_amount(deps: Deps, _env: Env) -> StdResult<Binary> {
    let config = config::read(deps.storage)?;

    to_binary(&resp::TotalDepositAmountResponse {
        amount: token::total_supply(deps, config.dp_token)?,
    })
}

pub fn claimable_reward(deps: Deps, env: Env) -> StdResult<Binary> {
    let reward = pool::claimable_rewards(deps, env)?;

    to_binary(&resp::ClaimableRewardResponse {
        amount: reward.amount,
        fee: reward.fee,
    })
}
