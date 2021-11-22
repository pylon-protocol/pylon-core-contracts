use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{attr, Api, Decimal, Env, MessageInfo, Response, Uint128};

use crate::error::ContractError;
use crate::executions::airdrop::instantiate;
use crate::executions::ExecuteResult;
use crate::state::airdrop::{Airdrop, Config};
use crate::state::state::State;
use crate::testing::{mock_deps, MockDeps, TEST_CREATOR, TEST_TOKEN, TEST_VOTER};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    start: u64,
    period: u64,
    reward_token: String,
    reward_amount: Uint128,
) -> ExecuteResult {
    instantiate(
        deps.as_mut(),
        env,
        info,
        start,
        period,
        reward_token,
        reward_amount,
    )
}

pub fn default(deps: &mut MockDeps, token: &str, amount: u128) -> (Env, MessageInfo, Response) {
    let env = mock_env();
    let info = mock_info(TEST_CREATOR, &[]);

    let response = exec(
        deps,
        env.clone(),
        info.clone(),
        env.block.time.seconds(),
        86400,
        token.to_string(),
        Uint128::from(amount),
    )
    .unwrap();

    (env, info, response)
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    let response = exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_CREATOR, &[]),
        100,
        200,
        TEST_TOKEN.to_string(),
        Uint128::from(1000u128),
    )
    .unwrap();
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "airdrop_instantiate"),
            attr("airdrop_id", 0.to_string()),
            attr("reward_token", TEST_TOKEN),
            attr("reward_amount", Uint128::from(1000u128))
        ]
    );

    let state = State::load(&deps.storage).unwrap();
    assert_eq!(state.total_airdrop_count, 1);
    assert_eq!(state.airdrop_update_candidates, vec![0]);

    let airdrop = Airdrop::load(&deps.storage, &0).unwrap();
    assert_eq!(
        airdrop.config,
        Config {
            start: 100,
            period: 200,
            reward_token: deps.api.addr_validate(TEST_TOKEN).unwrap(),
            reward_rate: Decimal::from_ratio(1000u128, 200u128)
        }
    )
}

#[test]
fn fail_unauthorized() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);

    match exec(
        &mut deps,
        mock_env(),
        mock_info(TEST_VOTER, &[]),
        100,
        200,
        TEST_TOKEN.to_string(),
        Uint128::from(1000u128),
    ) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
