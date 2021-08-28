use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{Env, Extern, HumanAddr, Uint128};
use pylon_core::pool_v2_msg::{HandleMsg, InitMsg};

use crate::contract;
use crate::testing::constants::{
    TEST_ADAPTER, TEST_BENEFICIARY, TEST_FACTORY, TEST_POOL_ID, TEST_POOL_NAME, TEST_TOKEN_CODE_ID,
    TEST_TOKEN_POOL, TEST_TOKEN_POOL_SUPPLY, TEST_TOKEN_YIELD, TEST_TOKEN_YIELD_SUPPLY,
};
use crate::testing::mock_querier::CustomMockQuerier;
use cw20::TokenInfoResponse;

pub fn init_msg() -> InitMsg {
    InitMsg {
        pool_id: TEST_POOL_ID,
        pool_name: TEST_POOL_NAME.to_string(),
        beneficiary: HumanAddr::from(TEST_BENEFICIARY),
        yield_adapter: HumanAddr::from(TEST_ADAPTER),
        dp_code_id: TEST_TOKEN_CODE_ID,
    }
}

pub fn initialize(mut deps: &mut Extern<MockStorage, MockApi, CustomMockQuerier>) -> Env {
    let env = mock_env(TEST_FACTORY, &[]);
    let msg = init_msg();
    let _res = contract::init(&mut deps, env.clone(), msg).expect("testing: contract initialized");

    let msg = HandleMsg::RegisterDPToken {};
    let _res = contract::handle(&mut deps, mock_env(TEST_TOKEN_POOL, &[]), msg)
        .expect("testing: dp token address registered");

    return env;
}
