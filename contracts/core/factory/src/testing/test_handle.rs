use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{log, to_binary, Api, CanonicalAddr, CosmosMsg, HumanAddr, WasmMsg};
use pylon_core::factory_msg::HandleMsg;
use pylon_core::pool_v2_msg::InitMsg;
use std::ops::Add;
use std::str::FromStr;

use crate::contract;
use crate::state::{adapter, config, pool, state};
use crate::testing::constants::{
    TEST_BENEFICIARY, TEST_CREATOR, TEST_POOL, TEST_USER, TEST_YIELD_ADAPTER,
};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn handle_configure_nothing() {
    let mut deps = mock_dependencies(20, &[]);
    let env = utils::initialize(&mut deps);

    let prev_config = config::read(&deps.storage).unwrap();

    let msg = HandleMsg::Configure {
        owner: None,
        pool_code_id: None,
        token_code_id: None,
        fee_rate: None,
        fee_collector: None,
    };
    let res = contract::handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len(), "should be empty");
    assert_eq!(None, res.data, "should be None");

    let next_config = config::read(&deps.storage).unwrap();
    assert_eq!(prev_config, next_config); // check nothing changed
}

#[test]
fn handle_configure_many_as_possible() {
    let mut deps = mock_dependencies(20, &[]);
    let env = utils::initialize(&mut deps);

    let new_config = config::Config {
        owner: deps
            .api
            .canonical_address(&HumanAddr::from("new_owner"))
            .unwrap(),
        pool_code_id: 3333,
        token_code_id: 4444,
        fee_rate: Decimal256::from_str("10.0").unwrap(),
        fee_collector: deps
            .api
            .canonical_address(&HumanAddr::from("new_fee_collector"))
            .unwrap(),
    };

    let new_owner = deps.api.human_address(&new_config.owner).unwrap();
    let new_fee_collector = deps.api.human_address(&new_config.fee_collector).unwrap();

    let msg = HandleMsg::Configure {
        owner: Option::from(new_owner),
        pool_code_id: Option::from(new_config.pool_code_id),
        token_code_id: Option::from(new_config.token_code_id),
        fee_rate: Option::from(new_config.fee_rate),
        fee_collector: Option::from(new_fee_collector),
    };
    let res = contract::handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len(), "should be empty");
    assert_eq!(None, res.data, "should be None");

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(config, new_config);
}

#[test]
fn handle_configure_with_non_owner() {
    let mut deps = mock_dependencies(20, &[]);
    let _env = utils::initialize(&mut deps);

    let msg = HandleMsg::Configure {
        owner: None,
        pool_code_id: None,
        token_code_id: None,
        fee_rate: None,
        fee_collector: None,
    };
    let _res = contract::handle(&mut deps, mock_env(TEST_USER, &[]), msg)
        .expect_err("should fail if non-owner tries to configure");
}

#[test]
fn handle_create_pool_and_register() {
    let mut deps = mock_dependencies(20, &[]);
    let env = utils::initialize(&mut deps);

    let config = config::read(&deps.storage).unwrap();
    let prev_state = state::read(&deps.storage).unwrap();

    let msg = HandleMsg::CreatePool {
        pool_name: TEST_POOL.into(),
        beneficiary: HumanAddr::from(TEST_BENEFICIARY),
        yield_adapter: HumanAddr::from(TEST_YIELD_ADAPTER),
    };
    let res = contract::handle(&mut deps, env.clone(), msg).unwrap();
    assert_eq!(
        res.messages,
        vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: config.pool_code_id.clone(),
            msg: to_binary(&InitMsg {
                pool_id: prev_state.next_pool_id.clone(),
                pool_name: TEST_POOL.into(),
                beneficiary: HumanAddr::from(TEST_BENEFICIARY),
                yield_adapter: HumanAddr::from(TEST_YIELD_ADAPTER),
                dp_code_id: config.token_code_id.clone(),
            })
            .unwrap(),
            send: vec![],
            label: None
        })]
    );
    assert_eq!(
        res.log,
        vec![
            log("action", "create_pool"),
            log("sender", TEST_CREATOR),
            log("pool_id", prev_state.next_pool_id.clone()),
        ]
    );
    assert_eq!(None, res.data, "should be None");

    let next_state = state::read(&deps.storage).unwrap();
    assert_eq!(next_state.next_pool_id, prev_state.next_pool_id.add(1));

    let pool = pool::read(&deps.storage, prev_state.next_pool_id.clone()).unwrap();
    assert_eq!(pool.status, pool::Status::Ready);

    let msg = HandleMsg::RegisterPool {
        pool_id: prev_state.next_pool_id.clone(),
    };
    let res = contract::handle(&mut deps, mock_env(TEST_POOL, &[]), msg).unwrap();
    assert_eq!(0, res.messages.len(), "should be empty");
    assert_eq!(None, res.data, "should be None");

    let pool = pool::read(&deps.storage, prev_state.next_pool_id.clone()).unwrap();
    assert_eq!(pool.status, pool::Status::Deployed);
    assert_eq!(
        pool.address,
        deps.api
            .canonical_address(&HumanAddr::from(TEST_POOL))
            .unwrap()
    );
}

#[test]
fn handle_create_pool_with_unregistered_adapter() {
    let mut deps = mock_dependencies(20, &[]);
    let env = utils::initialize(&mut deps);

    let msg = HandleMsg::CreatePool {
        pool_name: TEST_POOL.into(),
        beneficiary: HumanAddr::from(TEST_BENEFICIARY),
        yield_adapter: HumanAddr::from("mock_adapter"),
    };
    let _res = contract::handle(&mut deps, env.clone(), msg)
        .expect_err("should fail if given adapter address is not registered");
}

#[test]
fn handle_register_pool_which_is_not_ready() {
    let mut deps = mock_dependencies(20, &[]);
    let env = utils::initialize(&mut deps);

    let msg = HandleMsg::RegisterPool { pool_id: 1234 };
    let _res = contract::handle(&mut deps, mock_env(TEST_POOL, &[]), msg)
        .expect_err("should fail if given pool id is not ready");
}

#[test]
fn handle_register_unregister_adapter() {
    let mut deps = mock_dependencies(20, &[]);
    let env = utils::initialize(&mut deps);

    let new_adapter = HumanAddr::from("new_adapter");

    let msg = HandleMsg::RegisterAdapter {
        address: new_adapter.clone(),
    };
    let res = contract::handle(&mut deps, env.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len(), "should be empty");
    assert_eq!(None, res.data, "should be None");

    let adapter = adapter::read(
        &deps.storage,
        deps.api.canonical_address(&new_adapter).unwrap(),
    )
    .unwrap();
    assert_eq!(
        adapter.address,
        deps.api.canonical_address(&new_adapter).unwrap()
    );

    let msg = HandleMsg::UnregisterAdapter {
        address: new_adapter.clone(),
    };
    let res = contract::handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len(), "should be empty");
    assert_eq!(None, res.data, "should be None");

    let adapter = adapter::read(
        &deps.storage,
        deps.api.canonical_address(&new_adapter).unwrap(),
    )
    .unwrap();
    assert_eq!(adapter.address, CanonicalAddr::default());
}

#[test]
fn handle_register_adapter_with_non_owner() {
    let mut deps = mock_dependencies(20, &[]);
    let _env = utils::initialize(&mut deps);
    let user = mock_env(TEST_USER, &[]);

    let msg = HandleMsg::RegisterAdapter {
        address: HumanAddr::from("mock_adapter"),
    };
    let _res = contract::handle(&mut deps, user, msg)
        .expect_err("should fail if non-owner tries to register");
}

#[test]
fn handle_unregister_adapter_with_non_owner() {
    let mut deps = mock_dependencies(20, &[]);
    let _env = utils::initialize(&mut deps);
    let user = mock_env(TEST_USER, &[]);

    let msg = HandleMsg::UnregisterAdapter {
        address: HumanAddr::from(TEST_YIELD_ADAPTER),
    };
    let _res = contract::handle(&mut deps, user, msg)
        .expect_err("should fail if non-owner tries to unregister");
}
