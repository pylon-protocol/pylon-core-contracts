use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{Api, HumanAddr, InitResponse};
use std::str::FromStr;

use crate::contract;
use crate::state::{config, state, vpool};
use crate::testing::constants::{
    TEST_ADDITIONAL_CAP_PER_TOKEN, TEST_BASE_PRICE, TEST_BENEFICIARY, TEST_MAX_STAKE_AMOUNT,
    TEST_MAX_USER_CAP, TEST_MIN_STAKE_AMOUNT, TEST_MIN_USER_CAP, TEST_OWNER, TEST_POOL_LIQ_X,
    TEST_POOL_LIQ_Y, TEST_POOL_X_DENOM, TEST_POOL_Y_ADDR, TEST_STAKING, TEST_TOTAL_SALE_AMOUNT,
};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = utils::init_msg();
    let env = mock_env(TEST_OWNER, &[]);
    let res = contract::init(&mut deps, env, msg).unwrap();
    assert_eq!(res, InitResponse::default());

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config,
        config::Config {
            this: HumanAddr::from(MOCK_CONTRACT_ADDR),
            owner: HumanAddr::from(TEST_OWNER),
            beneficiary: HumanAddr::from(TEST_BENEFICIARY),
            base_price: Decimal256::from_str(TEST_BASE_PRICE).unwrap(),
            min_user_cap: Uint256::from(TEST_MIN_USER_CAP),
            max_user_cap: Uint256::from(TEST_MAX_USER_CAP),
            staking_contract: HumanAddr::from(TEST_STAKING),
            min_stake_amount: Uint256::from(TEST_MIN_STAKE_AMOUNT),
            max_stake_amount: Uint256::from(TEST_MAX_STAKE_AMOUNT),
            additional_cap_per_token: Decimal256::from_str(TEST_ADDITIONAL_CAP_PER_TOKEN).unwrap(),
            total_sale_amount: Uint256::from(TEST_TOTAL_SALE_AMOUNT),
            start: 0,
            finish: 1
        }
    );

    let state = state::read(&deps.storage).unwrap();
    assert_eq!(
        state,
        state::State {
            total_supply: Uint256::zero()
        }
    );

    let vpool = vpool::read(&deps.storage).unwrap();
    assert_eq!(
        vpool,
        vpool::VirtualPool {
            x_denom: TEST_POOL_X_DENOM.to_string(),
            y_addr: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_POOL_Y_ADDR))
                .unwrap(),
            liq_x: Uint256::from(TEST_POOL_LIQ_X),
            liq_y: Uint256::from(TEST_POOL_LIQ_Y)
        }
    )
}
