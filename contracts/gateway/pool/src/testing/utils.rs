use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{Env, Extern, HumanAddr, StdError};
use pylon_gateway::pool_msg::InitMsg;

use crate::contract;
use crate::testing::constants::{
    TEST_OWNER, TEST_POOL_CLIFF, TEST_POOL_PERIOD, TEST_POOL_REWARD_RATE, TEST_POOL_START,
    TEST_REWARD_TOKEN, TEST_SHARE_TOKEN,
};
use cosmwasm_bignumber::Decimal256;
use std::str::FromStr;

pub fn init_msg() -> InitMsg {
    InitMsg {
        start: TEST_POOL_START,
        period: TEST_POOL_PERIOD,
        cliff: TEST_POOL_CLIFF,
        reward_rate: Decimal256::from_str(TEST_POOL_REWARD_RATE).unwrap(),
        share_token: HumanAddr::from(TEST_SHARE_TOKEN),
        reward_token: HumanAddr::from(TEST_REWARD_TOKEN),
    }
}

pub fn initialize(mut deps: &mut Extern<MockStorage, MockApi, MockQuerier>) -> Env {
    let env = mock_env(TEST_OWNER, &[]);
    let msg = init_msg();
    contract::init(&mut deps, env.clone(), msg).expect("testing: contract initialized");

    env
}

pub fn assert_generic_err(func_name: &str, err: StdError) {
    assert!(matches!(err, StdError::GenericErr { .. }));
    if let StdError::GenericErr { msg, backtrace } = err {
        println!("{} | msg: {}", func_name, msg);
        if let Some(backtrace) = backtrace {
            println!("{} | backtrace: {}", func_name, backtrace)
        }
    }
}
