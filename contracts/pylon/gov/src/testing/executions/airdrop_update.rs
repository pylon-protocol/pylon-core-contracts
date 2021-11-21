use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, Api, Decimal, Env, MessageInfo, Uint128};

use crate::executions::airdrop::update;
use crate::executions::ExecuteResult;
use crate::state::airdrop::{Airdrop, Reward};
use crate::testing::{mock_deps, mock_env_height, MockDeps, TEST_TOKEN, TEST_VOTER, VOTING_TOKEN};

pub fn exec(
    deps: &mut MockDeps,
    env: Env,
    info: MessageInfo,
    target: Option<String>,
) -> ExecuteResult {
    update(deps.as_mut(), env, info, target)
}

#[test]
fn success() {
    let mut deps = mock_deps();
    super::instantiate::default(&mut deps);
    let (env, _, _) = super::airdrop_instantiate::default(&mut deps, TEST_TOKEN, 86400);

    let response = exec(
        &mut deps,
        env.clone(),
        mock_info(TEST_VOTER, &[]),
        Some(TEST_VOTER.to_string()),
    )
    .unwrap();
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "airdrop_update"),
            attr("updated", "[0]".to_string())
        ]
    );

    let reward = Reward::load(
        &deps.storage,
        &deps.api.addr_validate(TEST_VOTER).unwrap(),
        &0,
    )
    .unwrap();
    assert_eq!(reward.reward_per_token_paid, Decimal::zero());

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

    let response = exec(
        &mut deps,
        mock_env_height(env.block.height, env.block.time.seconds() + 100),
        mock_info(TEST_VOTER, &[]),
        Some(TEST_VOTER.to_string()),
    )
    .unwrap();
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "airdrop_update"),
            attr("updated", "[0]".to_string())
        ]
    );

    let airdrop = Airdrop::load(&deps.storage, &0).unwrap();
    assert_eq!(
        airdrop.state.last_update_time,
        env.block.time.seconds() + 100
    );
    assert_eq!(airdrop.state.reward_per_token_stored, Decimal::one());

    let reward = Reward::load(
        &deps.storage,
        &deps.api.addr_validate(TEST_VOTER).unwrap(),
        &0,
    )
    .unwrap();
    assert_eq!(reward.reward, Uint128::from(100u128));
    assert_eq!(reward.reward_per_token_paid, Decimal::one());

    let response = exec(
        &mut deps,
        mock_env_height(
            env.block.height,
            env.block.time.seconds() + airdrop.config.period * 2,
        ),
        mock_info(TEST_VOTER, &[]),
        Some(TEST_VOTER.to_string()),
    )
    .unwrap();
    assert_eq!(
        response.attributes,
        vec![
            attr("action", "airdrop_update"),
            attr("updated", "[0]".to_string())
        ]
    );

    let airdrop = Airdrop::load(&deps.storage, &0).unwrap();
    assert_eq!(
        airdrop.state.last_update_time,
        env.block.time.seconds() + airdrop.config.period
    );
    assert_eq!(
        airdrop.state.reward_per_token_stored,
        Decimal::from_ratio(86400u128, 100u128)
    );

    let reward = Reward::load(
        &deps.storage,
        &deps.api.addr_validate(TEST_VOTER).unwrap(),
        &0,
    )
    .unwrap();
    assert_eq!(reward.reward, Uint128::from(86400u128));
    assert_eq!(
        reward.reward_per_token_paid,
        Decimal::from_ratio(86400u128, 100u128)
    );
}
