use cosmwasm_std::{
    log, to_binary, Api, CosmosMsg, Decimal, Env, Extern, HandleResponse, HandleResult, HumanAddr,
    Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::Cw20HandleMsg;
use pylon_token::gov::{ExecuteMsg, PollStatus, VoteOption, VoterInfo};

use crate::querier::gov;
use crate::state::{bank, config, poll, state};
use std::ops::Sub;

const MIN_TITLE_LENGTH: usize = 4;
const MAX_TITLE_LENGTH: usize = 64;
const MIN_DESC_LENGTH: usize = 4;
const MAX_DESC_LENGTH: usize = 1024;
const MIN_LINK_LENGTH: usize = 12;
const MAX_LINK_LENGTH: usize = 128;

// validate_title returns an error if the title is invalid
fn validate_title(title: &str) -> StdResult<()> {
    if title.len() < MIN_TITLE_LENGTH {
        Err(StdError::generic_err("Title too short"))
    } else if title.len() > MAX_TITLE_LENGTH {
        Err(StdError::generic_err("Title too long"))
    } else {
        Ok(())
    }
}

/// validate_description returns an error if the description is invalid
fn validate_description(description: &str) -> StdResult<()> {
    if description.len() < MIN_DESC_LENGTH {
        Err(StdError::generic_err("Description too short"))
    } else if description.len() > MAX_DESC_LENGTH {
        Err(StdError::generic_err("Description too long"))
    } else {
        Ok(())
    }
}

/// validate_link returns an error if the link is invalid
fn validate_link(link: &Option<String>) -> StdResult<()> {
    if let Some(link) = link {
        if link.len() < MIN_LINK_LENGTH {
            Err(StdError::generic_err("Link too short"))
        } else if link.len() > MAX_LINK_LENGTH {
            Err(StdError::generic_err("Link too long"))
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
/// create a new poll
pub fn create<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    proposer: HumanAddr,
    deposit_amount: Uint128,
    title: String,
    description: String,
    link: Option<String>,
    execute_msgs: Option<Vec<ExecuteMsg>>,
) -> StdResult<HandleResponse> {
    validate_title(&title)?;
    validate_description(&description)?;
    validate_link(&link)?;

    let config = config::read(&deps.storage).load()?;
    if deposit_amount < config.proposal_deposit {
        return Err(StdError::generic_err(format!(
            "Must deposit more than {} token",
            config.proposal_deposit
        )));
    }

    let mut state = state::store(&mut deps.storage).load()?;
    let poll_id = state.poll_count + 1;

    // Increase poll count & total deposit amount
    state.poll_count += 1;
    state.total_deposit += deposit_amount;

    let mut data_list: Vec<poll::ExecuteData> = vec![];
    let all_execute_data = if let Some(exe_msgs) = execute_msgs {
        for msgs in exe_msgs {
            let execute_data = poll::ExecuteData {
                order: msgs.order,
                contract: deps.api.canonical_address(&msgs.contract)?,
                msg: msgs.msg,
            };
            data_list.push(execute_data)
        }
        Some(data_list)
    } else {
        None
    };

    let sender_address_raw = deps.api.canonical_address(&proposer)?;
    let new_poll = poll::Poll {
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

    poll::store(&mut deps.storage).save(&poll_id.to_be_bytes(), &new_poll)?;
    poll::store_indexer(&mut deps.storage, &PollStatus::InProgress)
        .save(&poll_id.to_be_bytes(), &true)?;

    state::store(&mut deps.storage).save(&state)?;

    let r = HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "create_poll"),
            log(
                "creator",
                deps.api.human_address(&new_poll.creator)?.as_str(),
            ),
            log("poll_id", &poll_id.to_string()),
            log("end_height", new_poll.end_height),
        ],
        data: None,
    };
    Ok(r)
}

/*
 * Ends a poll.
 */
pub fn end<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    poll_id: u64,
) -> HandleResult {
    let mut a_poll = poll::store(&mut deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::InProgress {
        return Err(StdError::generic_err("Poll is not in progress"));
    }

    if a_poll.end_height > env.block.height {
        return Err(StdError::generic_err("Voting period has not expired"));
    }

    let no = a_poll.no_votes.u128();
    let yes = a_poll.yes_votes.u128();

    let tallied_weight = yes + no;

    let mut poll_status = PollStatus::Rejected;
    let mut rejected_reason = "";
    let mut passed = false;

    let mut messages: Vec<CosmosMsg> = vec![];
    let config = config::read(&deps.storage).load()?;
    let mut state = state::read(&deps.storage).load()?;

    let (quorum, staked_weight) = if state.total_share.u128() == 0 {
        (Decimal::zero(), Uint128::zero())
    } else if let Some(staked_amount) = a_poll.staked_amount {
        (
            Decimal::from_ratio(tallied_weight, staked_amount),
            staked_amount,
        )
    } else {
        let token_balance = gov::load_token_balance(
            deps,
            &deps.api.human_address(&config.pylon_token)?,
            &state.contract_addr,
        )?;
        let staked_weight = token_balance.sub(state.total_deposit)?;

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
                contract_addr: deps.api.human_address(&config.pylon_token)?,
                send: vec![],
                msg: to_binary(&Cw20HandleMsg::Transfer {
                    recipient: deps.api.human_address(&a_poll.creator)?,
                    amount: a_poll.deposit_amount,
                })?,
            }))
        }
    }

    // Decrease total deposit amount
    state.total_deposit = (state.total_deposit - a_poll.deposit_amount)?;
    state::store(&mut deps.storage).save(&state)?;

    // Update poll indexer
    poll::store_indexer(&mut deps.storage, &PollStatus::InProgress)
        .remove(&a_poll.id.to_be_bytes());
    poll::store_indexer(&mut deps.storage, &poll_status).save(&a_poll.id.to_be_bytes(), &true)?;

    // Update poll status
    a_poll.status = poll_status;
    a_poll.total_balance_at_end_poll = Some(staked_weight);
    poll::store(&mut deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "end_poll"),
            log("poll_id", &poll_id.to_string()),
            log("rejected_reason", rejected_reason),
            log("passed", &passed.to_string()),
        ],
        data: None,
    })
}

/*
 * Execute a msg of passed poll.
 */
pub fn execute<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    poll_id: u64,
) -> HandleResult {
    let config = config::read(&deps.storage).load()?;
    let mut a_poll = poll::store(&mut deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::Passed {
        return Err(StdError::generic_err("Poll is not in passed status"));
    }

    if a_poll.end_height + config.timelock_period > env.block.height {
        return Err(StdError::generic_err("Timelock period has not expired"));
    }

    poll::store_indexer(&mut deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes());
    poll::store_indexer(&mut deps.storage, &PollStatus::Executed)
        .save(&poll_id.to_be_bytes(), &true)?;

    a_poll.status = PollStatus::Executed;
    poll::store(&mut deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if let Some(all_msgs) = a_poll.execute_data {
        let mut msgs = all_msgs;
        msgs.sort();
        for msg in msgs {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&msg.contract)?,
                msg: msg.msg,
                send: vec![],
            }))
        }
    } else {
        return Err(StdError::generic_err("The poll does not have execute_data"));
    }

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "execute_poll"),
            log("poll_id", poll_id.to_string()),
        ],
        data: None,
    })
}

/// ExpirePoll is used to make the poll as expired state for querying purpose
pub fn expire<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    poll_id: u64,
) -> HandleResult {
    let config = config::read(&deps.storage).load()?;
    let mut a_poll = poll::store(&mut deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::Passed {
        return Err(StdError::generic_err("Poll is not in passed status"));
    }

    if a_poll.execute_data.is_none() {
        return Err(StdError::generic_err(
            "Cannot make a text proposal to expired state",
        ));
    }

    if a_poll.end_height + config.expiration_period > env.block.height {
        return Err(StdError::generic_err("Expire height has not been reached"));
    }

    poll::store_indexer(&mut deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes());
    poll::store_indexer(&mut deps.storage, &PollStatus::Expired)
        .save(&poll_id.to_be_bytes(), &true)?;

    a_poll.status = PollStatus::Expired;
    poll::store(&mut deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "expire_poll"),
            log("poll_id", poll_id.to_string()),
        ],
        data: None,
    })
}

/// SnapshotPoll is used to take a snapshot of the staked amount for quorum calculation
pub fn snapshot<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    poll_id: u64,
) -> HandleResult {
    let config = config::read(&deps.storage).load()?;
    let mut a_poll = poll::store(&mut deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::InProgress {
        return Err(StdError::generic_err("Poll is not in progress"));
    }

    let time_to_end = a_poll.end_height - env.block.height;

    if time_to_end > config.snapshot_period {
        return Err(StdError::generic_err("Cannot snapshot at this height"));
    }

    if a_poll.staked_amount.is_some() {
        return Err(StdError::generic_err("Snapshot has already occurred"));
    }

    // store the current staked amount for quorum calculation
    let state = state::store(&mut deps.storage).load()?;

    let token_balance = gov::load_token_balance(
        deps,
        &deps.api.human_address(&config.pylon_token)?,
        &state.contract_addr,
    )?;
    let staked_amount = token_balance.sub(state.total_deposit)?;

    a_poll.staked_amount = Some(staked_amount);

    poll::store(&mut deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "snapshot_poll"),
            log("poll_id", poll_id.to_string()),
            log("staked_amount", staked_amount),
        ],
        data: None,
    })
}

pub fn cast_vote<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    poll_id: u64,
    vote: VoteOption,
    amount: Uint128,
) -> HandleResult {
    let sender_address_raw = deps.api.canonical_address(&env.message.sender)?;
    let config = config::read(&deps.storage).load()?;
    let state = state::read(&deps.storage).load()?;
    if poll_id == 0 || state.poll_count < poll_id {
        return Err(StdError::generic_err("Poll does not exist"));
    }

    let mut a_poll = poll::store(&mut deps.storage).load(&poll_id.to_be_bytes())?;
    if a_poll.status != PollStatus::InProgress || env.block.height > a_poll.end_height {
        return Err(StdError::generic_err("Poll is not in progress"));
    }

    // Check the voter already has a vote on the poll
    if poll::read_voter(&deps.storage, poll_id)
        .load(sender_address_raw.as_slice())
        .is_ok()
    {
        return Err(StdError::generic_err("User has already voted."));
    }

    let key = &sender_address_raw.as_slice();
    let mut token_manager = bank::read(&deps.storage).may_load(key)?.unwrap_or_default();

    // convert share to amount
    let token_balance = gov::load_token_balance(
        deps,
        &deps.api.human_address(&config.pylon_token)?,
        &state.contract_addr,
    )?;
    let total_share = state.total_share;
    let total_balance = token_balance.sub(state.total_deposit)?;

    if token_manager
        .share
        .multiply_ratio(total_balance, total_share)
        < amount
    {
        return Err(StdError::generic_err(
            "User does not have enough staked tokens.",
        ));
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
    bank::store(&mut deps.storage).save(key, &token_manager)?;

    // store poll voter && and update poll data
    poll::store_voter(&mut deps.storage, poll_id)
        .save(sender_address_raw.as_slice(), &vote_info)?;

    // processing snapshot
    let time_to_end = a_poll.end_height - env.block.height;

    if time_to_end < config.snapshot_period && a_poll.staked_amount.is_none() {
        a_poll.staked_amount = Some(total_balance);
    }

    poll::store(&mut deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    let log = vec![
        log("action", "cast_vote"),
        log("poll_id", &poll_id.to_string()),
        log("amount", &amount.to_string()),
        log("voter", &env.message.sender.as_str()),
        log("vote_option", vote_info.vote),
    ];

    let r = HandleResponse {
        messages: vec![],
        log,
        data: None,
    };
    Ok(r)
}
