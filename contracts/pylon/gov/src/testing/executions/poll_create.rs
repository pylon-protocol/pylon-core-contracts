use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, to_binary, Deps, Env, MessageInfo, Response, StdError, Uint128};
use cw20::Cw20ExecuteMsg;
use pylon_token::gov_msg::PollExecuteMsg;

use crate::error::ContractError;
use crate::executions::poll::create;
use crate::executions::ExecuteResult;
use crate::state::poll::PollCategory;
use crate::state::state::State;
use crate::testing::{mock_deps, MockDeps, LONG_STRING, SHORT_STRING, TEST_CREATOR, VOTING_TOKEN};

pub struct Message {
    pub proposer: String,
    pub deposit: Uint128,
    pub title: String,
    pub category: PollCategory,
    pub description: String,
    pub link: Option<String>,
    pub execute_msg: Option<Vec<PollExecuteMsg>>,
}

pub fn exec(deps: &mut MockDeps, env: Env, _info: MessageInfo, msg: Message) -> ExecuteResult {
    create(
        deps.as_mut(),
        env,
        msg.proposer,
        msg.deposit,
        msg.title,
        msg.category,
        msg.description,
        msg.link,
        msg.execute_msg,
    )
}

pub fn default(deps: &mut MockDeps) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = mock_info(TEST_CREATOR, &[]);

    let response = exec(deps, env.clone(), info.clone(), default_msg()).unwrap();

    (env, info, response)
}

pub fn default_msg() -> Message {
    Message {
        proposer: TEST_CREATOR.to_string(),
        deposit: super::instantiate::default_msg().proposal_deposit,
        title: "test".to_string(),
        category: PollCategory::Core,
        description: "test".to_string(),
        link: None,
        execute_msg: None,
    }
}

#[allow(dead_code)]
pub fn default_exec_msgs() -> Vec<PollExecuteMsg> {
    let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(123),
    })
    .unwrap();

    let exec_msg_bz2 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(12),
    })
    .unwrap();

    let exec_msg_bz3 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(1),
    })
    .unwrap();

    vec![
        PollExecuteMsg {
            order: 3u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz3,
        },
        PollExecuteMsg {
            order: 2u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz2,
        },
        PollExecuteMsg {
            order: 1u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz,
        },
    ]
}

// helper to confirm the expected create_poll response
pub fn assert_create_poll_result(
    poll_id: u64,
    end_height: u64,
    creator: &str,
    execute_res: Response,
    deps: Deps,
) {
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "create_poll"),
            attr("creator", creator),
            attr("poll_id", poll_id.to_string()),
            attr("end_height", end_height.to_string()),
        ]
    );

    //confirm poll count
    let state = State::load(deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            poll_count: 1,
            total_share: Uint128::zero(),
            total_deposit: super::instantiate::default_msg().proposal_deposit,
            total_airdrop_count: 0,
            airdrop_update_candidates: vec![]
        }
    );
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let (env, _, response) = default(&mut deps);
    assert_create_poll_result(
        1,
        env.block.height + super::instantiate::default_msg().voting_period,
        TEST_CREATOR,
        response,
        deps.as_ref(),
    );
}

#[test]
fn fail_invalid_title() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let mut msg = default_msg();
    msg.title = SHORT_STRING.to_string();
    match exec(&mut deps, mock_env(), mock_info(VOTING_TOKEN, &[]), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "Title too short")
        }
        Err(_) => panic!("Unknown error"),
    }

    let mut msg = default_msg();
    msg.title = LONG_STRING.to_string();
    match exec(&mut deps, mock_env(), mock_info(VOTING_TOKEN, &[]), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "Title too long")
        }
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn fail_invalid_description() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let mut msg = default_msg();
    msg.description = SHORT_STRING.to_string();
    match exec(&mut deps, mock_env(), mock_info(VOTING_TOKEN, &[]), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "Description too short")
        }
        Err(_) => panic!("Unknown error"),
    }

    let mut msg = default_msg();
    msg.description = LONG_STRING.to_string();
    match exec(&mut deps, mock_env(), mock_info(VOTING_TOKEN, &[]), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "Description too long")
        }
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn fail_invalid_link() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let mut msg = default_msg();
    msg.link = Some("http://hih".to_string());
    match exec(&mut deps, mock_env(), mock_info(VOTING_TOKEN, &[]), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "Link too short")
        }
        Err(_) => panic!("Unknown error"),
    }

    let mut msg = default_msg();
    msg.link = Some(LONG_STRING.to_string());
    match exec(&mut deps, mock_env(), mock_info(VOTING_TOKEN, &[]), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "Link too long")
        }
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn fail_invalid_deposit() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let default_deposit = super::instantiate::default_msg().proposal_deposit;
    let mut msg = default_msg();
    msg.deposit = default_deposit - Uint128::from(1u128);
    match exec(&mut deps, mock_env(), mock_info(VOTING_TOKEN, &[]), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InsufficientProposalDeposit(amount)) => {
            assert_eq!(amount, default_deposit.u128());
        }
        Err(_) => panic!("Unknown error"),
    }
}
