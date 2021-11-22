use cosmwasm_std::{
    to_binary, CanonicalAddr, CosmosMsg, DepsMut, Env, MessageInfo, Response, Storage, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terraswap::querier::query_token_balance;

use crate::error::ContractError;
use crate::executions::ExecuteResult;
use crate::state::bank::TokenManager;
use crate::state::config::Config;
use crate::state::poll::{Poll, PollStatus, VoterInfo};
use crate::state::state::State;

// INTERNAL
pub fn stake_voting_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Uint128,
) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "staking");

    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if amount.is_zero() {
        return Err(ContractError::InsufficientFunds {});
    }

    let sender_address_raw = deps.api.addr_canonicalize(sender.as_str())?;
    let mut token_manager = TokenManager::load(deps.storage, &sender_address_raw)?;
    let config = Config::load(deps.storage)?;
    let mut state = State::load(deps.storage)?;

    // balance already increased, so subtract deposit amount
    let total_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.pylon_token)?,
        env.contract.address,
    )?
    .checked_sub(state.total_deposit + amount)?;

    let share = if total_balance.is_zero() || state.total_share.is_zero() {
        amount
    } else {
        amount.multiply_ratio(state.total_share, total_balance)
    };

    token_manager.share += share;
    state.total_share += share;

    State::save(deps.storage, &state)?;
    TokenManager::save(deps.storage, &sender_address_raw, &token_manager)?;

    Ok(response.add_attributes(vec![
        ("sender", sender.as_str()),
        ("share", share.to_string().as_str()),
        ("amount", amount.to_string().as_str()),
    ]))
}

// INTERNAL
pub fn withdraw_voting_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: String,
    amount: Option<Uint128>,
) -> ExecuteResult {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let sender_address_raw = deps.api.addr_canonicalize(sender.as_str())?;
    let mut token_manager = TokenManager::load(deps.storage, &sender_address_raw)?;

    if !token_manager.share.is_zero() {
        let config = Config::load(deps.storage)?;
        let mut state = State::load(deps.storage)?;

        // Load total share & total balance except proposal deposit amount
        let total_share = state.total_share.u128();
        let total_balance = query_token_balance(
            &deps.querier,
            deps.api.addr_humanize(&config.pylon_token)?,
            env.contract.address,
        )?
        .checked_sub(state.total_deposit)?
        .u128();

        let locked_balance =
            compute_locked_balance(deps.storage, &mut token_manager, &sender_address_raw);
        let locked_share = locked_balance * total_share / total_balance;
        let user_share = token_manager.share.u128();

        let withdraw_share = amount
            .map(|v| std::cmp::max(v.multiply_ratio(total_share, total_balance).u128(), 1u128))
            .unwrap_or_else(|| user_share - locked_share);
        let withdraw_amount = amount
            .map(|v| v.u128())
            .unwrap_or_else(|| withdraw_share * total_balance / total_share);

        if locked_share + withdraw_share > user_share {
            Err(ContractError::InvalidWithdrawAmount {})
        } else {
            let share = user_share - withdraw_share;
            token_manager.share = Uint128::from(share);

            TokenManager::save(deps.storage, &sender_address_raw, &token_manager)?;

            state.total_share = Uint128::from(total_share - withdraw_share);
            State::save(deps.storage, &state)?;

            send_tokens(
                deps,
                &config.pylon_token,
                &sender_address_raw,
                withdraw_amount,
                "withdraw",
            )
        }
    } else {
        Err(ContractError::NothingStaked {})
    }
}

// removes not in-progress poll voter info & unlock tokens
// and returns the largest locked amount in participated polls.
fn compute_locked_balance(
    storage: &mut dyn Storage,
    token_manager: &mut TokenManager,
    voter: &CanonicalAddr,
) -> u128 {
    token_manager.locked_balance.retain(|(poll_id, _)| {
        let poll = Poll::load(storage, poll_id).unwrap();

        if poll.status != PollStatus::InProgress {
            // remove voter info from the poll
            VoterInfo::remove(storage, poll_id, voter);
        }

        poll.status == PollStatus::InProgress
    });

    token_manager
        .locked_balance
        .iter()
        .map(|(_, v)| v.balance.u128())
        .max()
        .unwrap_or_default()
}

fn send_tokens(
    deps: DepsMut,
    asset_token: &CanonicalAddr,
    recipient: &CanonicalAddr,
    amount: u128,
    action: &str,
) -> ExecuteResult {
    let contract_human = deps.api.addr_humanize(asset_token)?.to_string();
    let recipient_human = deps.api.addr_humanize(recipient)?.to_string();

    Ok(Response::new()
        .add_messages(vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_human,
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient_human.clone(),
                amount: Uint128::from(amount),
            })?,
            funds: vec![],
        })])
        .add_attributes(vec![
            ("action", action),
            ("recipient", recipient_human.as_str()),
            ("amount", amount.to_string().as_str()),
        ]))
}
