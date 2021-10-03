use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{Env, MessageInfo, OwnedDeps};
use pylon_gateway::swap_msg::InstantiateMsg;
use std::str::FromStr;

use crate::contract;
use crate::testing::constants::{
    TEST_BASE_PRICE, TEST_BENEFICIARY, TEST_OWNER, TEST_POOL_LIQ_X, TEST_POOL_LIQ_Y,
    TEST_POOL_X_DENOM, TEST_POOL_Y_ADDR, TEST_TOTAL_SALE_AMOUNT,
};
use crate::testing::mock_querier::CustomMockQuerier;

pub fn init_msg() -> InstantiateMsg {
    InstantiateMsg {
        beneficiary: TEST_BENEFICIARY.to_string(),
        pool_x_denom: TEST_POOL_X_DENOM.to_string(),
        pool_y_addr: TEST_POOL_Y_ADDR.to_string(),
        pool_liq_x: Uint256::from(TEST_POOL_LIQ_X),
        pool_liq_y: Uint256::from(TEST_POOL_LIQ_Y),
        price: Decimal256::from_str(TEST_BASE_PRICE).unwrap(),
        cap_strategy: None,
        distribution_strategy: vec![],
        whitelist_enabled: false,
        start: 0,
        period: 1,
        swap_pool_size: Uint256::from(TEST_TOTAL_SALE_AMOUNT),
    }
}

pub fn initialize(
    deps: &mut OwnedDeps<MockStorage, MockApi, CustomMockQuerier>,
) -> (Env, MessageInfo) {
    let env = mock_env();
    let info = mock_info(TEST_OWNER, &[]);
    let msg = init_msg();
    let _res = contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: contract initialized");

    (env, info)
}
