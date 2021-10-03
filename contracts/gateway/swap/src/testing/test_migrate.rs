use cosmwasm_bignumber::Decimal256;
use cosmwasm_std::Response;
use cosmwasm_storage::ReadonlySingleton;
use pylon_gateway::swap_msg::MigrateMsg;
use std::str::FromStr;

use crate::contract;
use crate::handler::migrate::NewRefundConfig;
use crate::state::config::KEY_CONFIG;
use crate::testing::constants::{TEST_OWNER, TEST_POOL_X_DENOM, TEST_PRICE};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
pub fn migrate() {
    let mut deps = mock_dependencies(&[]);
    let (env, _) = utils::initialize(&mut deps);

    let msg = MigrateMsg::Refund {};
    let res = contract::migrate(deps.as_mut(), env, msg).unwrap();
    assert_eq!(res, Response::default());

    let config: NewRefundConfig = ReadonlySingleton::new(deps.as_ref().storage, KEY_CONFIG)
        .load()
        .unwrap();
    assert_eq!(
        config,
        NewRefundConfig {
            manager: TEST_OWNER.to_string(),
            swap_price: Decimal256::from_str(TEST_PRICE).unwrap(),
            refund_denom: TEST_POOL_X_DENOM.to_string(),
        }
    )
}
