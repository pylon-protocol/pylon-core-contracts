use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{to_binary, Attribute, CosmosMsg, SubMsg, WasmMsg};
use pylon_core::factory_msg::{ConfigureMsg, ExecuteMsg};
use pylon_core::pool_v2_msg::InstantiateMsg;
use pylon_core::test_constant::*;
use std::ops::Add;
use std::str::FromStr;

use crate::contract;
use crate::state::{adapter, config, pool, state};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn handle_configure_nothing() {
    let mut deps = mock_dependencies(&[]);
    let (env, info) = utils::initialize(&mut deps);

    let prev_config = config::read(deps.as_ref().storage).unwrap();

    let msg = ExecuteMsg::Configure(ConfigureMsg {
        owner: None,
        pool_code_id: None,
        token_code_id: None,
        fee_rate: None,
        fee_collector: None,
    });
    let res = contract::execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len(), "should be empty");
    assert_eq!(None, res.data, "should be None");

    let next_config = config::read(deps.as_ref().storage).unwrap();
    assert_eq!(prev_config, next_config); // check nothing changed
}

#[test]
fn handle_configure_many_as_possible() {
    let mut deps = mock_dependencies(&[]);
    let (env, info) = utils::initialize(&mut deps);

    let new_config = config::Config {
        owner: "new_owner".to_string(),
        pool_code_id: 3333,
        token_code_id: 4444,
        fee_rate: Decimal256::from_str("10.0").unwrap(),
        fee_collector: "new_fee_collector".to_string(),
    };

    let msg = ExecuteMsg::Configure(ConfigureMsg {
        owner: Option::from(new_config.owner.clone()),
        pool_code_id: Option::from(new_config.pool_code_id),
        token_code_id: Option::from(new_config.token_code_id),
        fee_rate: Option::from(new_config.fee_rate),
        fee_collector: Option::from(new_config.fee_collector.clone()),
    });
    let res = contract::execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len(), "should be empty");
    assert_eq!(None, res.data, "should be None");

    let config = config::read(deps.as_ref().storage).unwrap();
    assert_eq!(config, new_config);
}

#[test]
fn handle_configure_with_non_owner() {
    let mut deps = mock_dependencies(&[]);
    let _env = utils::initialize(&mut deps);

    let msg = ExecuteMsg::Configure(ConfigureMsg {
        owner: None,
        pool_code_id: None,
        token_code_id: None,
        fee_rate: None,
        fee_collector: None,
    });
    let _res = contract::execute(deps.as_mut(), mock_env(), mock_info(TEST_USER, &[]), msg)
        .expect_err("should fail if non-owner tries to configure");
}

#[test]
fn handle_create_pool_and_register() {
    let mut deps = mock_dependencies(&[]);
    let (env, info) = utils::initialize(&mut deps);

    let config = config::read(deps.as_ref().storage).unwrap();
    let prev_state = state::read(deps.as_ref().storage).unwrap();

    let msg = ExecuteMsg::CreatePool {
        pool_name: TEST_POOL.into(),
        beneficiary: TEST_BENEFICIARY.to_string(),
        yield_adapter: TEST_ADAPTER.to_string(),
    };
    let res = contract::execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: None,
            code_id: config.pool_code_id,
            msg: to_binary(&InstantiateMsg {
                pool_id: prev_state.next_pool_id,
                pool_name: TEST_POOL.into(),
                beneficiary: TEST_BENEFICIARY.to_string(),
                yield_adapter: TEST_ADAPTER.to_string(),
                dp_code_id: config.token_code_id,
            })
            .unwrap(),
            label: "".to_string(),
            funds: vec![]
        }))]
    );
    assert_eq!(
        res.attributes,
        vec![
            Attribute::new("action", "create_pool"),
            Attribute::new("sender", TEST_CREATOR.to_string()),
            Attribute::new("pool_id", prev_state.next_pool_id.to_string()),
        ]
    );
    assert_eq!(None, res.data, "should be None");

    let next_state = state::read(deps.as_ref().storage).unwrap();
    assert_eq!(next_state.next_pool_id, prev_state.next_pool_id.add(1));

    let pool = pool::read(deps.as_ref().storage, prev_state.next_pool_id).unwrap();
    assert_eq!(pool.status, pool::Status::Ready);

    let msg = ExecuteMsg::RegisterPool {
        pool_id: prev_state.next_pool_id,
    };
    let res = contract::execute(deps.as_mut(), mock_env(), mock_info(TEST_POOL, &[]), msg).unwrap();
    assert_eq!(0, res.messages.len(), "should be empty");
    assert_eq!(None, res.data, "should be None");

    let pool = pool::read(deps.as_ref().storage, prev_state.next_pool_id).unwrap();
    assert_eq!(pool.status, pool::Status::Deployed);
    assert_eq!(pool.address, TEST_POOL.to_string());
}

#[test]
fn handle_create_pool_with_unregistered_adapter() {
    let mut deps = mock_dependencies(&[]);
    let (env, info) = utils::initialize(&mut deps);

    let msg = ExecuteMsg::CreatePool {
        pool_name: TEST_POOL.into(),
        beneficiary: TEST_BENEFICIARY.to_string(),
        yield_adapter: "mock_adapter".to_string(),
    };
    let _res = contract::execute(deps.as_mut(), env, info, msg)
        .expect_err("should fail if given adapter address is not registered");
}

#[test]
fn handle_register_pool_which_is_not_ready() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);

    let msg = ExecuteMsg::RegisterPool { pool_id: 1234 };
    let _res = contract::execute(deps.as_mut(), mock_env(), mock_info(TEST_POOL, &[]), msg)
        .expect_err("should fail if given pool id is not ready");
}

#[test]
fn handle_register_unregister_adapter() {
    let mut deps = mock_dependencies(&[]);
    let (env, info) = utils::initialize(&mut deps);

    let new_adapter = "new_adapter".to_string();

    let msg = ExecuteMsg::RegisterAdapter {
        address: new_adapter.clone(),
    };
    let res = contract::execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len(), "should be empty");
    assert_eq!(None, res.data, "should be None");

    let adapter = adapter::read(deps.as_ref().storage, new_adapter.clone()).unwrap();
    assert_eq!(adapter.address, new_adapter);

    let msg = ExecuteMsg::UnregisterAdapter {
        address: new_adapter.clone(),
    };
    let res = contract::execute(deps.as_mut(), env, info, msg).unwrap();
    assert_eq!(0, res.messages.len(), "should be empty");
    assert_eq!(None, res.data, "should be None");

    let adapter = adapter::read(deps.as_ref().storage, new_adapter).unwrap();
    assert_eq!(adapter.address, "".to_string());
}

#[test]
fn handle_register_adapter_with_non_owner() {
    let mut deps = mock_dependencies(&[]);
    let (env, _info) = utils::initialize(&mut deps);
    let user = mock_info(TEST_USER, &[]);

    let msg = ExecuteMsg::RegisterAdapter {
        address: "mock_adapter".to_string(),
    };
    let _res = contract::execute(deps.as_mut(), env, user, msg)
        .expect_err("should fail if non-owner tries to register");
}

#[test]
fn handle_unregister_adapter_with_non_owner() {
    let mut deps = mock_dependencies(&[]);
    let _env = utils::initialize(&mut deps);
    let user = mock_info(TEST_USER, &[]);

    let msg = ExecuteMsg::UnregisterAdapter {
        address: TEST_ADAPTER.to_string(),
    };
    let _res = contract::execute(deps.as_mut(), mock_env(), user, msg)
        .expect_err("should fail if non-owner tries to unregister");
}
