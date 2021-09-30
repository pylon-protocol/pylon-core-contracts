use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{Env, MessageInfo, OwnedDeps};
use pylon_core::factory_msg::{ExecuteMsg, InstantiateMsg};
use pylon_core::test_constant::*;

use crate::contract;
use crate::testing::mock_querier::CustomMockQuerier;

pub fn init_msg() -> InstantiateMsg {
    InstantiateMsg {
        pool_code_id: TEST_POOL_CODE_ID,
        token_code_id: TEST_TOKEN_CODE_ID,
        fee_rate: Decimal256::percent(TEST_FACTORY_FEE_RATE),
        fee_collector: TEST_FEE_COLLECTOR.to_string(),
    }
}

pub fn initialize(
    deps: &mut OwnedDeps<MockStorage, MockApi, CustomMockQuerier>,
) -> (Env, MessageInfo) {
    let env = mock_env();
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = init_msg();
    let _res = contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: contract initialized");

    let msg = ExecuteMsg::RegisterAdapter {
        address: TEST_ADAPTER.to_string(),
    };
    let _res = contract::execute(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: adapter registered");

    (env, info)
}
