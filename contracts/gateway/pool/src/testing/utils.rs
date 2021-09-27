use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{Env, HumanAddr, MessageInfo, OwnedDeps, StdError};
use pylon_gateway::pool_msg::InstantiateMsg;

use crate::contract;
use crate::error::ContractError;
use crate::testing::constants::{
    TEST_OWNER, TEST_POOL_CLIFF, TEST_POOL_PERIOD, TEST_POOL_REWARD_RATE, TEST_POOL_START,
    TEST_REWARD_TOKEN, TEST_SHARE_TOKEN,
};
use cosmwasm_bignumber::Decimal256;
use std::str::FromStr;

pub fn init_msg() -> InstantiateMsg {
    InstantiateMsg {
        start: TEST_POOL_START,
        period: TEST_POOL_PERIOD,
        cliff: TEST_POOL_CLIFF,
        reward_rate: Decimal256::from_str(TEST_POOL_REWARD_RATE).unwrap(),
        share_token: TEST_SHARE_TOKEN.to_string(),
        reward_token: TEST_REWARD_TOKEN.to_string(),
    }
}

pub fn initialize(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) -> (Env, MessageInfo) {
    let env = mock_env();
    let info = mock_info(TEST_OWNER, &[]);
    let msg = init_msg();
    contract::instantiate(deps.as_mut(), env.clone(), info.clone(), msg)
        .expect("testing: contract initialized");

    (env, info)
}

pub fn assert_generic_err(func_name: &str, err: ContractError) {
    println!("{} | {:?}", func_name, err);
}
