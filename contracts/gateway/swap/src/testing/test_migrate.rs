use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::{HumanAddr, MigrateResponse};
use cosmwasm_storage::ReadonlySingleton;
use pylon_gateway::swap_msg::MigrateMsg;
use std::str::FromStr;

use crate::contract;
use crate::handler::migrate::NewConfig;
use crate::state::config::KEY_CONFIG;
use crate::testing::constants::{TEST_BASE_PRICE, TEST_OWNER, TEST_POOL_X_DENOM};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
pub fn migrate() {
    let mut deps = mock_dependencies(20, &[]);
    let env = utils::initialize(&mut deps);

    let msg = MigrateMsg {};
    let res = contract::migrate(&mut deps, env.clone(), msg.clone()).unwrap();
    assert_eq!(res, MigrateResponse::default());

    let config: NewConfig = ReadonlySingleton::new(&deps.storage, KEY_CONFIG)
        .load()
        .unwrap();
    assert_eq!(
        config,
        NewConfig {
            manager: HumanAddr::from(TEST_OWNER),
            refund_denom: TEST_POOL_X_DENOM.to_string(),
            base_price: Decimal256::from_str(TEST_BASE_PRICE).unwrap(),
        }
    )
}
