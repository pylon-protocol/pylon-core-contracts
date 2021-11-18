use cosmwasm_std::{
    attr, to_binary, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128,
    WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use pylon_token::gov_msg::{ExecuteMsg, PollExecuteMsg, PollMsg};
use terraswap::querier::query_token_balance;

use crate::constant::POLL_EXECUTE_REPLY_ID;
use crate::error::ContractError;
use crate::executions::ExecuteResult;
use crate::state::bank::TokenManager;
use crate::state::config::Config;
use crate::state::poll::{ExecuteData, Poll, PollCategory, PollStatus, VoteOption, VoterInfo};
use crate::state::state::State;

#[allow(clippy::too_many_arguments)]
/// create a new poll
pub fn create(
    deps: DepsMut,
    env: Env,
    proposer: String,
    deposit_amount: Uint128,
    title: String,
    category: PollCategory,
    description: String,
    link: Option<String>,
    execute_msgs: Option<Vec<PollExecuteMsg>>,
) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "create_poll");

    let config = Config::load(deps.storage)?;
    if deposit_amount < config.proposal_deposit {
        return Err(ContractError::InsufficientProposalDeposit(
            config.proposal_deposit.u128(),
        ));
    }

    let mut state = State::load(deps.storage)?;
    let poll_id = state.poll_count + 1;

    // Increase poll count & total deposit amount
    state.poll_count += 1;
    state.total_deposit += deposit_amount;

    let mut data_list: Vec<ExecuteData> = vec![];
    let all_execute_data = if let Some(exec_msgs) = execute_msgs {
        for msgs in exec_msgs {
            let execute_data = ExecuteData {
                order: msgs.order,
                contract: deps.api.addr_canonicalize(&msgs.contract)?,
                msg: msgs.msg,
            };
            data_list.push(execute_data)
        }
        Some(data_list)
    } else {
        None
    };

    let sender_address_raw = deps.api.addr_canonicalize(&proposer)?;
    let new_poll = Poll {
        id: poll_id,
        creator: sender_address_raw,
        status: PollStatus::InProgress,
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        end_height: env.block.height + config.voting_period,
        title,
        category,
        description,
        link,
        execute_data: all_execute_data,
        deposit_amount,
        total_balance_at_end_poll: None,
        staked_amount: None,
    };
    new_poll.validate()?;

    Poll::save(deps.storage, &poll_id, &new_poll)?;
    Poll::index_status(deps.storage, &poll_id, &PollStatus::InProgress)?;
    Poll::index_category(deps.storage, &poll_id, &new_poll.category)?;

    State::save(deps.storage, &state)?;

    Ok(response.add_attributes(vec![
        ("creator", proposer.as_str()),
        ("poll_id", &poll_id.to_string()),
        ("end_height", new_poll.end_height.to_string().as_str()),
    ]))
}

pub fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
    vote: VoteOption,
    amount: Uint128,
) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "cast_vote");

    let sender_address_raw = deps.api.addr_canonicalize(info.sender.as_str())?;
    let config = Config::load(deps.storage)?;
    let state = State::load(deps.storage)?;
    if poll_id == 0 || state.poll_count < poll_id {
        return Err(ContractError::PollNotFound {});
    }

    let mut poll = Poll::load(deps.storage, &poll_id)?;
    if poll.status != PollStatus::InProgress || env.block.height > poll.end_height {
        return Err(ContractError::PollNotInProgress {});
    }

    // Check the voter already has a vote on the poll
    if VoterInfo::load(deps.storage, &poll_id, &sender_address_raw).is_ok() {
        return Err(ContractError::AlreadyVoted {});
    }

    let mut token_manager = TokenManager::load(deps.storage, &sender_address_raw)?;

    // convert share to amount
    let total_share = state.total_share;
    let total_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.pylon_token)?,
        env.contract.address,
    )?
    .checked_sub(state.total_deposit)?;

    if token_manager
        .share
        .multiply_ratio(total_balance, total_share)
        < amount
    {
        return Err(ContractError::InsufficientStaked {});
    }

    // update tally info
    if VoteOption::Yes == vote {
        poll.yes_votes += amount;
    } else {
        poll.no_votes += amount;
    }

    let vote_info = VoterInfo {
        vote,
        balance: amount,
    };
    token_manager
        .locked_balance
        .push((poll_id, vote_info.clone()));
    TokenManager::save(deps.storage, &sender_address_raw, &token_manager)?;

    // store poll voter && and update poll data
    VoterInfo::save(deps.storage, &poll_id, &sender_address_raw, &vote_info)?;

    // processing snapshot
    let time_to_end = poll.end_height - env.block.height;

    if time_to_end < config.snapshot_period && poll.staked_amount.is_none() {
        poll.staked_amount = Some(total_balance);
    }

    Poll::save(deps.storage, &poll_id, &poll)?;

    Ok(response.add_attributes(vec![
        ("poll_id", poll_id.to_string().as_str()),
        ("amount", amount.to_string().as_str()),
        ("voter", info.sender.as_str()),
        ("vote_option", vote_info.vote.to_string().as_str()),
    ]))
}

/*
 * Execute a msgs of passed poll as one submsg to catch failures
 */
pub fn execute(deps: DepsMut, env: Env, poll_id: u64) -> ExecuteResult {
    let config = Config::load(deps.storage)?;
    let poll = Poll::load(deps.storage, &poll_id)?;

    if poll.status != PollStatus::Passed {
        return Err(ContractError::PollNotPassed {});
    }

    if poll.end_height + config.timelock_period > env.block.height {
        return Err(ContractError::TimelockNotExpired {});
    }

    Poll::save_temp_id(deps.storage, &poll.id)?;

    Ok(Response::new().add_submessage(SubMsg::reply_on_error(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.to_string(),
            msg: to_binary(&ExecuteMsg::Poll(PollMsg::ExecuteMsgs { poll_id }))?,
            funds: vec![],
        }),
        POLL_EXECUTE_REPLY_ID,
    )))
}

/*
 * Execute a msgs of a poll
 */
pub fn execute_messages(deps: DepsMut, env: Env, info: MessageInfo, poll_id: u64) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "execute_poll");

    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut poll = Poll::load(deps.storage, &poll_id)?;

    poll.status = PollStatus::Executed;

    Poll::deindex_status(deps.storage, &poll_id, &PollStatus::Passed);
    Poll::index_status(deps.storage, &poll_id, &PollStatus::Executed)?;
    Poll::save(deps.storage, &poll_id, &poll)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if let Some(all_msgs) = poll.execute_data {
        let mut msgs = all_msgs;
        msgs.sort();
        for msg in msgs {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&msg.contract)?.to_string(),
                msg: msg.msg,
                funds: vec![],
            }));
        }
    }

    Ok(response
        .add_messages(messages)
        .add_attributes(vec![("poll_id", poll_id.to_string().as_str())]))
}

/// SnapshotPoll is used to take a snapshot of the staked amount for quorum calculation
pub fn snapshot(deps: DepsMut, env: Env, _info: MessageInfo, poll_id: u64) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "snapshot_poll");

    let config = Config::load(deps.storage)?;
    let state = State::load(deps.storage)?;
    let staked_amount = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.pylon_token)?,
        env.contract.address,
    )?
    .checked_sub(state.total_deposit)?;

    let mut poll = Poll::load(deps.storage, &poll_id)?;
    if poll.status != PollStatus::InProgress {
        return Err(ContractError::PollNotInProgress {});
    }

    if poll.staked_amount.is_some() {
        return Err(ContractError::SnapshotAlreadyOccurred {});
    }

    let time_to_end = poll.end_height - env.block.height;
    if time_to_end > config.snapshot_period {
        return Err(ContractError::SnapshotHeight {});
    }

    poll.staked_amount = Some(staked_amount);

    Poll::save(deps.storage, &poll_id, &poll)?;

    Ok(response.add_attributes(vec![
        attr("poll_id", poll_id.to_string().as_str()),
        attr("staked_amount", staked_amount.to_string().as_str()),
    ]))
}

/*
 * Set the status of a poll to Failed if execute_poll fails
 */
pub fn fail(deps: DepsMut, poll_id: u64) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "fail_poll");

    let mut poll = Poll::load(deps.storage, &poll_id)?;

    poll.status = PollStatus::Failed;

    Poll::deindex_status(deps.storage, &poll_id, &PollStatus::Passed);
    Poll::index_status(deps.storage, &poll_id, &PollStatus::Failed)?;
    Poll::save(deps.storage, &poll_id, &poll)?;

    Ok(response.add_attributes(vec![("poll_id", poll_id.to_string().as_str())]))
}

/*
 * Ends a poll.
 */
pub fn end(deps: DepsMut, env: Env, poll_id: u64) -> ExecuteResult {
    let response = Response::new().add_attribute("action", "end_poll");

    let mut poll = Poll::load(deps.storage, &poll_id)?;
    if poll.status != PollStatus::InProgress {
        return Err(ContractError::PollNotInProgress {});
    }

    if poll.end_height > env.block.height {
        return Err(ContractError::PollVotingPeriod {});
    }

    let no = poll.no_votes.u128();
    let yes = poll.yes_votes.u128();
    let tallied_weight = yes + no;

    let mut poll_status = PollStatus::Rejected;
    let mut rejected_reason = "";
    let mut passed = false;

    let mut messages: Vec<CosmosMsg> = vec![];
    let config = Config::load(deps.storage)?;
    let mut state = State::load(deps.storage)?;

    let (quorum, staked_weight) = if state.total_share.u128() == 0 {
        (Decimal::zero(), Uint128::zero())
    } else if let Some(staked_amount) = poll.staked_amount {
        (
            Decimal::from_ratio(tallied_weight, staked_amount),
            staked_amount,
        )
    } else {
        let staked_weight = query_token_balance(
            &deps.querier,
            deps.api.addr_humanize(&config.pylon_token)?,
            env.contract.address,
        )?
        .checked_sub(state.total_deposit)?;

        (
            Decimal::from_ratio(tallied_weight, staked_weight),
            staked_weight,
        )
    };

    if tallied_weight == 0 || quorum < config.quorum {
        // Quorum: More than quorum of the total staked tokens at the end of the voting
        // period need to have participated in the vote.
        rejected_reason = "Quorum not reached";
    } else {
        if Decimal::from_ratio(yes, tallied_weight) > config.threshold {
            //Threshold: More than 50% of the tokens that participated in the vote
            // (after excluding “Abstain” votes) need to have voted in favor of the proposal (“Yes”).
            poll_status = PollStatus::Passed;
            passed = true;
        } else {
            rejected_reason = "Threshold not reached";
        }

        // Refunds deposit only when quorum is reached
        if !poll.deposit_amount.is_zero() {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.pylon_token)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: deps.api.addr_humanize(&poll.creator)?.to_string(),
                    amount: poll.deposit_amount,
                })?,
            }))
        }
    }

    // Decrease total deposit amount
    state.total_deposit = state.total_deposit.checked_sub(poll.deposit_amount)?;
    State::save(deps.storage, &state)?;

    // Update poll indexer
    Poll::deindex_status(deps.storage, &poll.id, &PollStatus::InProgress);
    Poll::index_status(deps.storage, &poll.id, &poll_status)?;

    // Update poll status
    poll.status = poll_status;
    poll.total_balance_at_end_poll = Some(staked_weight);
    Poll::save(deps.storage, &poll_id, &poll)?;

    Ok(response.add_messages(messages).add_attributes(vec![
        ("poll_id", &poll_id.to_string()),
        ("rejected_reason", &rejected_reason.to_string()),
        ("passed", &passed.to_string()),
    ]))
}
