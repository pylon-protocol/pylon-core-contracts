use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, Api, Env, MessageInfo, Uint128};

use crate::error::ContractError;
use crate::executions::airdrop::deallocate;
use crate::executions::ExecuteResult;
use crate::state::airdrop::Reward;
use crate::testing::{mock_deps, MockDeps, TEST_CREATOR, TEST_TOKEN, TEST_VOTER};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    airdrop_id: u64,
    recipient: String,
    deallocate_amount: Uint128,
) -> ExecuteResult {
    deallocate(
        deps.as_mut(),
        env,
        info,
        airdrop_id,
        recipient,
        deallocate_amount,
    )
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);
    super::airdrop_instantiate::default(&mut deps, TEST_TOKEN, 1000u128);
    super::airdrop_allocate::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_CREATOR, &[]),
        0,
        TEST_VOTER.to_string(),
        Uint128::from(1234u128),
    )
    .unwrap();

    let response = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_CREATOR, &[]),
        0,
        TEST_VOTER.to_string(),
        Uint128::from(34u128),
    )
    .unwrap();
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "airdrop_deallocate"),
            attr("airdrop_id", 0.to_string()),
            attr("recipient", TEST_VOTER),
            attr("amount", Uint128::from(34u128)),
        ]
    );

    let reward = Reward::load(
        &deps.storage,
        &deps.api.addr_validate(TEST_VOTER).unwrap(),
        &0u64,
    )
    .unwrap();
    assert_eq!(reward.reward, Uint128::from(1200u128));
}

#[test]
fn fail_unauthorized() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);
    super::airdrop_instantiate::default(&mut deps, TEST_TOKEN, 1000u128);
    super::airdrop_allocate::exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_CREATOR, &[]),
        0,
        TEST_VOTER.to_string(),
        Uint128::from(1234u128),
    )
    .unwrap();

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        0,
        TEST_VOTER.to_string(),
        Uint128::from(34u128),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
