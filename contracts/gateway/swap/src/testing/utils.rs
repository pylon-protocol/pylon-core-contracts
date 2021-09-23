use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{Env, Extern, HumanAddr};
use pylon_gateway::swap_msg::InitMsg;
use std::str::FromStr;

use crate::contract;
use crate::testing::constants::{
    TEST_ADDITIONAL_CAP_PER_TOKEN, TEST_BASE_PRICE, TEST_BENEFICIARY, TEST_MAX_STAKE_AMOUNT,
    TEST_MAX_USER_CAP, TEST_MIN_STAKE_AMOUNT, TEST_MIN_USER_CAP, TEST_OWNER, TEST_POOL_LIQ_X,
    TEST_POOL_LIQ_Y, TEST_POOL_X_DENOM, TEST_POOL_Y_ADDR, TEST_STAKING, TEST_TOTAL_SALE_AMOUNT,
};
use crate::testing::mock_querier::CustomMockQuerier;

pub fn init_msg() -> InitMsg {
    InitMsg {
        beneficiary: HumanAddr::from(TEST_BENEFICIARY),
        pool_x_denom: TEST_POOL_X_DENOM.to_string(),
        pool_y_addr: HumanAddr::from(TEST_POOL_Y_ADDR),
        pool_liq_x: Uint256::from(TEST_POOL_LIQ_X),
        pool_liq_y: Uint256::from(TEST_POOL_LIQ_Y),
        base_price: Decimal256::from_str(TEST_BASE_PRICE).unwrap(),
        min_user_cap: Uint256::from(TEST_MIN_USER_CAP),
        max_user_cap: Uint256::from(TEST_MAX_USER_CAP),
        staking_contract: HumanAddr::from(TEST_STAKING),
        min_stake_amount: Uint256::from(TEST_MIN_STAKE_AMOUNT),
        max_stake_amount: Uint256::from(TEST_MAX_STAKE_AMOUNT),
        additional_cap_per_token: Decimal256::from_str(TEST_ADDITIONAL_CAP_PER_TOKEN).unwrap(),
        total_sale_amount: Uint256::from(TEST_TOTAL_SALE_AMOUNT),
        start: 0,
        period: 1,
    }
}

pub fn initialize(mut deps: &mut Extern<MockStorage, MockApi, CustomMockQuerier>) -> Env {
    let env = mock_env(TEST_OWNER, &[]);
    let msg = init_msg();
    let _res = contract::init(&mut deps, env.clone(), msg).expect("testing: contract initialized");

    env
}
