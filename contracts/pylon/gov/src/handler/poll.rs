use cosmwasm_std::{
    attr, to_binary, Binary, CosmosMsg, Decimal, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use pylon_token::common::OrderBy;
use pylon_token::gov::{
    ExecuteMsg, PollExecuteMsg, PollMsg, PollResponse, PollStatus, PollsResponse, VoteOption,
    VoterInfo, VotersResponse, VotersResponseItem,
};
use terraswap::querier::query_token_balance;

use crate::constant::POLL_EXECUTE_REPLY_ID;
use crate::error::ContractError;
use crate::querier::poll::{poll_voters, polls, store_tmp_poll_id};
use crate::state::bank::{bank_r, bank_w};
use crate::state::config::config_r;
use crate::state::poll::{
    poll_indexer_w, poll_r, poll_voter_r, poll_voter_w, poll_w, ExecuteData, Poll,
};
use crate::state::state::{state_r, state_w};

/**
 * Poll Lifecycle
 * 1. CreatePoll
 * 2. CastVote
 * 3. ExecutePoll
 * 3-A. ExecutePollMsgs (Internal)
 * 3-B. FailPoll (Reply)
 * 4. SnapshotPoll
 * 5. EndPoll
 */
pub fn handle_poll_msg(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: PollMsg,
) -> Result<Response, ContractError> {
    match msg {
        // CreatePoll
        PollMsg::CastVote {
            poll_id,
            vote,
            amount,
        } => cast_vote(deps, env, info, poll_id, vote, amount),
        PollMsg::Execute { poll_id } => execute_poll(deps, env, poll_id),
        PollMsg::ExecuteMsgs { poll_id } => execute_poll_messages(deps, env, info, poll_id),
        PollMsg::Snapshot { poll_id } => snapshot_poll(deps, env, poll_id),
        // FailPoll
        PollMsg::End { poll_id } => end_poll(deps, env, poll_id),
    }
}

#[allow(clippy::too_many_arguments)]
/// create a new poll
pub fn create_poll(
    deps: DepsMut,
    env: Env,
    proposer: String,
    deposit_amount: Uint128,
    title: String,
    description: String,
    link: Option<String>,
    execute_msgs: Option<Vec<PollExecuteMsg>>,
) -> Result<Response, ContractError> {
    let config = config_r(deps.storage).load()?;
    if deposit_amount < config.proposal_deposit {
        return Err(ContractError::InsufficientProposalDeposit(
            config.proposal_deposit.u128(),
        ));
    }

    let mut state = state_r(deps.storage).load()?;
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
        description,
        link,
        execute_data: all_execute_data,
        deposit_amount,
        total_balance_at_end_poll: None,
        staked_amount: None,
    };
    new_poll.validate()?;

    poll_w(deps.storage).save(&poll_id.to_be_bytes(), &new_poll)?;
    poll_indexer_w(deps.storage, &PollStatus::InProgress).save(&poll_id.to_be_bytes(), &true)?;

    state_w(deps.storage).save(&state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "create_poll"),
        (
            "creator",
            deps.api
                .addr_humanize(&new_poll.creator)?
                .to_string()
                .as_str(),
        ),
        ("poll_id", &poll_id.to_string()),
        ("end_height", new_poll.end_height.to_string().as_str()),
    ]))
}

fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
    vote: VoteOption,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender_address_raw = deps.api.addr_canonicalize(info.sender.as_str())?;
    let config = config_r(deps.storage).load()?;
    let state = state_r(deps.storage).load()?;
    if poll_id == 0 || state.poll_count < poll_id {
        return Err(ContractError::PollNotFound {});
    }

    let mut a_poll = poll_r(deps.storage).load(&poll_id.to_be_bytes())?;
    if a_poll.status != PollStatus::InProgress || env.block.height > a_poll.end_height {
        return Err(ContractError::PollNotInProgress {});
    }

    // Check the voter already has a vote on the poll
    if poll_voter_r(deps.storage, poll_id)
        .load(sender_address_raw.as_slice())
        .is_ok()
    {
        return Err(ContractError::AlreadyVoted {});
    }

    let key = &sender_address_raw.as_slice();
    let mut token_manager = bank_r(deps.storage).may_load(key)?.unwrap_or_default();

    // convert share to amount
    let total_share = state.total_share;
    let total_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.pylon_token)?,
        deps.api.addr_humanize(&state.contract_addr)?,
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
        a_poll.yes_votes += amount;
    } else {
        a_poll.no_votes += amount;
    }

    let vote_info = VoterInfo {
        vote,
        balance: amount,
    };
    token_manager
        .locked_balance
        .push((poll_id, vote_info.clone()));
    bank_w(deps.storage).save(key, &token_manager)?;

    // store poll voter && and update poll data
    poll_voter_w(deps.storage, poll_id).save(sender_address_raw.as_slice(), &vote_info)?;

    // processing snapshot
    let time_to_end = a_poll.end_height - env.block.height;

    if time_to_end < config.snapshot_period && a_poll.staked_amount.is_none() {
        a_poll.staked_amount = Some(total_balance);
    }

    poll_w(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "cast_vote"),
        ("poll_id", poll_id.to_string().as_str()),
        ("amount", amount.to_string().as_str()),
        ("voter", info.sender.as_str()),
        ("vote_option", vote_info.vote.to_string().as_str()),
    ]))
}

/*
 * Execute a msgs of passed poll as one submsg to catch failures
 */
fn execute_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let config = config_r(deps.storage).load()?;
    let a_poll = poll_r(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::Passed {
        return Err(ContractError::PollNotPassed {});
    }

    if a_poll.end_height + config.timelock_period > env.block.height {
        return Err(ContractError::TimelockNotExpired {});
    }

    store_tmp_poll_id(deps.storage, a_poll.id)?;

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
fn execute_poll_messages(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
) -> Result<Response, ContractError> {
    if env.contract.address != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    let mut a_poll: Poll = poll_r(deps.storage).load(&poll_id.to_be_bytes())?;

    poll_indexer_w(deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes());
    poll_indexer_w(deps.storage, &PollStatus::Executed).save(&poll_id.to_be_bytes(), &true)?;

    a_poll.status = PollStatus::Executed;
    poll_w(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if let Some(all_msgs) = a_poll.execute_data {
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

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "execute_poll"),
        ("poll_id", poll_id.to_string().as_str()),
    ]))
}

/// SnapshotPoll is used to take a snapshot of the staked amount for quorum calculation
fn snapshot_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let config = config_r(deps.storage).load()?;
    let mut a_poll = poll_r(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::InProgress {
        return Err(ContractError::PollNotInProgress {});
    }

    let time_to_end = a_poll.end_height - env.block.height;

    if time_to_end > config.snapshot_period {
        return Err(ContractError::SnapshotHeight {});
    }

    if a_poll.staked_amount.is_some() {
        return Err(ContractError::SnapshotAlreadyOccurred {});
    }

    // store the current staked amount for quorum calculation
    let state = state_r(deps.storage).load()?;

    let staked_amount = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.pylon_token)?,
        deps.api.addr_humanize(&state.contract_addr)?,
    )?
    .checked_sub(state.total_deposit)?;

    a_poll.staked_amount = Some(staked_amount);

    poll_w(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "snapshot_poll"),
        attr("poll_id", poll_id.to_string().as_str()),
        attr("staked_amount", staked_amount.to_string().as_str()),
    ]))
}

/*
 * Set the status of a poll to Failed if execute_poll fails
 */
pub fn fail_poll(deps: DepsMut, poll_id: u64) -> Result<Response, ContractError> {
    let mut a_poll: Poll = poll_r(deps.storage).load(&poll_id.to_be_bytes())?;

    poll_indexer_w(deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes());
    poll_indexer_w(deps.storage, &PollStatus::Failed).save(&poll_id.to_be_bytes(), &true)?;

    a_poll.status = PollStatus::Failed;
    poll_w(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "fail_poll"),
        ("poll_id", poll_id.to_string().as_str()),
    ]))
}

/*
 * Ends a poll.
 */
fn end_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let mut a_poll = poll_r(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::InProgress {
        return Err(ContractError::PollNotInProgress {});
    }

    if a_poll.end_height > env.block.height {
        return Err(ContractError::PollVotingPeriod {});
    }

    let no = a_poll.no_votes.u128();
    let yes = a_poll.yes_votes.u128();

    let tallied_weight = yes + no;

    let mut poll_status = PollStatus::Rejected;
    let mut rejected_reason = "";
    let mut passed = false;

    let mut messages: Vec<CosmosMsg> = vec![];
    let config = config_r(deps.storage).load()?;
    let mut state = state_r(deps.storage).load()?;

    let (quorum, staked_weight) = if state.total_share.u128() == 0 {
        (Decimal::zero(), Uint128::zero())
    } else if let Some(staked_amount) = a_poll.staked_amount {
        (
            Decimal::from_ratio(tallied_weight, staked_amount),
            staked_amount,
        )
    } else {
        let staked_weight = query_token_balance(
            &deps.querier,
            deps.api.addr_humanize(&config.pylon_token)?,
            deps.api.addr_humanize(&state.contract_addr)?,
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
        if !a_poll.deposit_amount.is_zero() {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.pylon_token)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: deps.api.addr_humanize(&a_poll.creator)?.to_string(),
                    amount: a_poll.deposit_amount,
                })?,
            }))
        }
    }

    // Decrease total deposit amount
    state.total_deposit = state.total_deposit.checked_sub(a_poll.deposit_amount)?;
    state_w(deps.storage).save(&state)?;

    // Update poll indexer
    poll_indexer_w(deps.storage, &PollStatus::InProgress).remove(&a_poll.id.to_be_bytes());
    poll_indexer_w(deps.storage, &poll_status).save(&a_poll.id.to_be_bytes(), &true)?;

    // Update poll status
    a_poll.status = poll_status;
    a_poll.total_balance_at_end_poll = Some(staked_weight);
    poll_w(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "end_poll"),
        ("poll_id", &poll_id.to_string()),
        ("rejected_reason", rejected_reason),
        ("passed", &passed.to_string()),
    ]))
}

pub fn query_poll(deps: Deps, poll_id: u64) -> Result<Binary, ContractError> {
    let poll = match poll_r(deps.storage).may_load(&poll_id.to_be_bytes())? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::PollNotFound {}),
    }
    .unwrap();

    let mut data_list: Vec<PollExecuteMsg> = vec![];

    Ok(to_binary(&PollResponse {
        id: poll.id,
        creator: deps.api.addr_humanize(&poll.creator)?.to_string(),
        status: poll.status,
        end_height: poll.end_height,
        title: poll.title,
        description: poll.description,
        link: poll.link,
        deposit_amount: poll.deposit_amount,
        execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
            for msg in exe_msgs {
                let execute_data = PollExecuteMsg {
                    order: msg.order,
                    contract: deps.api.addr_humanize(&msg.contract)?.to_string(),
                    msg: msg.msg,
                };
                data_list.push(execute_data)
            }
            Some(data_list)
        } else {
            None
        },
        yes_votes: poll.yes_votes,
        no_votes: poll.no_votes,
        staked_amount: poll.staked_amount,
        total_balance_at_end_poll: poll.total_balance_at_end_poll,
    })?)
}

pub fn query_polls(
    deps: Deps,
    filter: Option<PollStatus>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> Result<Binary, ContractError> {
    let polls = polls(deps.storage, filter, start_after, limit, order_by)?;

    let poll_responses: StdResult<Vec<PollResponse>> = polls
        .iter()
        .map(|poll| {
            Ok(PollResponse {
                id: poll.id,
                creator: deps.api.addr_humanize(&poll.creator)?.to_string(),
                status: poll.status.clone(),
                end_height: poll.end_height,
                title: poll.title.to_string(),
                description: poll.description.to_string(),
                link: poll.link.clone(),
                deposit_amount: poll.deposit_amount,
                execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
                    let mut data_list: Vec<PollExecuteMsg> = vec![];

                    for msg in exe_msgs {
                        let execute_data = PollExecuteMsg {
                            order: msg.order,
                            contract: deps.api.addr_humanize(&msg.contract)?.to_string(),
                            msg: msg.msg,
                        };
                        data_list.push(execute_data)
                    }
                    Some(data_list)
                } else {
                    None
                },
                yes_votes: poll.yes_votes,
                no_votes: poll.no_votes,
                staked_amount: poll.staked_amount,
                total_balance_at_end_poll: poll.total_balance_at_end_poll,
            })
        })
        .collect();

    Ok(to_binary(&PollsResponse {
        polls: poll_responses?,
    })?)
}

pub fn query_voters(
    deps: Deps,
    poll_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> Result<Binary, ContractError> {
    let poll = match poll_r(deps.storage).may_load(&poll_id.to_be_bytes())? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::PollNotFound {}),
    }
    .unwrap();

    let voters = if poll.status != PollStatus::InProgress {
        vec![]
    } else if let Some(start_after) = start_after {
        poll_voters(
            deps.storage,
            poll_id,
            Some(deps.api.addr_canonicalize(&start_after)?),
            limit,
            order_by,
        )?
    } else {
        poll_voters(deps.storage, poll_id, None, limit, order_by)?
    };

    let voters_response: StdResult<Vec<VotersResponseItem>> = voters
        .iter()
        .map(|voter_info| {
            Ok(VotersResponseItem {
                voter: deps.api.addr_humanize(&voter_info.0)?.to_string(),
                vote: voter_info.1.vote.clone(),
                balance: voter_info.1.balance,
            })
        })
        .collect();

    Ok(to_binary(&VotersResponse {
        voters: voters_response?,
    })?)
}
