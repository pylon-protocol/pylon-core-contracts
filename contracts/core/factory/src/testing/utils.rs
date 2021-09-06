use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{Env, Extern, HumanAddr};
use pylon_core::factory_msg::{HandleMsg, InitMsg};
use std::str::FromStr;

use crate::contract;
use crate::testing::constants::{
    TEST_CREATOR, TEST_FEE_COLLECTOR, TEST_FEE_RATE, TEST_POOL_CODE_ID, TEST_TOKEN_CODE_ID,
    TEST_YIELD_ADAPTER,
};
use crate::testing::mock_querier::CustomMockQuerier;

pub fn init_msg() -> InitMsg {
    InitMsg {
        pool_code_id: TEST_POOL_CODE_ID,
        token_code_id: TEST_TOKEN_CODE_ID,
        fee_rate: Decimal256::from_str(TEST_FEE_RATE).unwrap(),
        fee_collector: HumanAddr::from(TEST_FEE_COLLECTOR),
    }
}

pub fn initialize(mut deps: &mut Extern<MockStorage, MockApi, CustomMockQuerier>) -> Env {
    let env = mock_env(TEST_CREATOR, &[]);
    let msg = init_msg();
    let _res = contract::init(&mut deps, env.clone(), msg).expect("testing: contract initialized");

    let msg = HandleMsg::RegisterAdapter {
        address: HumanAddr::from(TEST_YIELD_ADAPTER),
    };
    let _res = contract::handle(&mut deps, env.clone(), msg).expect("testing: adapter registered");

    env
}
