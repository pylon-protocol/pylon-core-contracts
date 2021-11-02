use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, coins, Uint128};
use pylon_token::gov_msg::{ExecuteMsg, PollMsg};

use crate::contract;
use crate::error::ContractError;
use crate::testing::constants::*;
use crate::testing::message::create_poll_msg;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils::mock_instantiate;

#[test]
fn snapshot_poll() {
    let stake_amount = 1000;

    let mut deps = mock_dependencies(&coins(100, VOTING_TOKEN));
    mock_instantiate(deps.as_mut());

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);
    let mut creator_env = mock_env();
    let creator_info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = contract::execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "create_poll"),
            attr("creator", TEST_CREATOR),
            attr("poll_id", "1"),
            attr("end_height", "32345"),
        ]
    );

    //must not be executed
    let snapshot_err = contract::execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        ExecuteMsg::Poll(PollMsg::Snapshot { poll_id: 1 }),
    )
    .unwrap_err();
    assert_eq!(ContractError::SnapshotHeight {}, snapshot_err);

    // change time
    creator_env.block.height = 32345 - 10;

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from((stake_amount + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    let fix_res = contract::execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        ExecuteMsg::Poll(PollMsg::Snapshot { poll_id: 1 }),
    )
    .unwrap();

    assert_eq!(
        fix_res.attributes,
        vec![
            attr("action", "snapshot_poll"),
            attr("poll_id", "1"),
            attr("staked_amount", stake_amount.to_string().as_str()),
        ]
    );

    //must not be executed
    let snapshot_error = contract::execute(
        deps.as_mut(),
        creator_env,
        creator_info,
        ExecuteMsg::Poll(PollMsg::Snapshot { poll_id: 1 }),
    )
    .unwrap_err();
    assert_eq!(ContractError::SnapshotAlreadyOccurred {}, snapshot_error);
}
