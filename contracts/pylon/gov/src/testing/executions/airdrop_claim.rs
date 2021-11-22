use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Env, MessageInfo, Uint128};

use crate::executions::airdrop::claim_internal;
use crate::executions::ExecuteResult;
use crate::testing::{mock_deps, mock_env_height, MockDeps, TEST_TOKEN, TEST_VOTER, VOTING_TOKEN};

#[allow(dead_code)]
pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    sender: String,
    airdrop_id: u64,
) -> ExecuteResult {
    claim_internal(deps.as_mut(), env, info, sender, airdrop_id)
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);
    let (env, _, _) = super::airdrop_instantiate::default(&mut deps, TEST_TOKEN, 86400);

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(100u128))],
    )]);

    super::staking_deposit::exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        Uint128::from(100u128),
    )
    .unwrap();

    super::airdrop_update::exec(
        &mut deps,
        mock_env_height(env.block.height, env.block.time.seconds() + 86400 * 2),
        mock_info(TEST_VOTER, &[]),
        Some(TEST_VOTER.to_string()),
    )
    .unwrap();

    let response = exec(
        &mut deps,
        mock_env(),
        mock_info(MOCK_CONTRACT_ADDR, &[]),
        TEST_VOTER.to_string(),
        0,
    )
    .unwrap();
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "airdrop_claim"),
            attr("target", TEST_VOTER),
            attr("token", TEST_TOKEN),
            attr("amount", Uint128::from(86400u128))
        ]
    );
}
