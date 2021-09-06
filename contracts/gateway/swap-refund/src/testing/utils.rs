use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::testing::{mock_env, MockApi, MockQuerier, MockStorage};
use cosmwasm_std::{Env, Extern, HumanAddr};
use std::str::FromStr;

use crate::contract;
use crate::testing::constants::{TEST_BASE_PRICE, TEST_MANAGER, TEST_OWNER, TEST_REFUND_DENOM};

pub fn init_msg() -> contract::InitMsg {
    contract::InitMsg {
        manager: HumanAddr::from(TEST_MANAGER),
        refund_denom: TEST_REFUND_DENOM.to_string(),
        base_price: Decimal256::from_str(TEST_BASE_PRICE).unwrap(),
    }
}

pub fn initialize(mut deps: &mut Extern<MockStorage, MockApi, MockQuerier>) -> Env {
    let env = mock_env(TEST_OWNER, &[]);
    let msg = init_msg();
    let _res = contract::init(&mut deps, env.clone(), msg).expect("testing: contract initialized");

    env
}
