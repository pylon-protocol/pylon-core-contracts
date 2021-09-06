use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{Api, HumanAddr};
use std::str::FromStr;

use crate::contract;
use crate::contract::HandleMsg;
use crate::handler::query::Buyer;
use crate::state::{config, user, user_state};
use crate::testing::constants::{TEST_MANAGER, TEST_NON_MANAGER, TEST_OWNER};
use crate::testing::utils;
use std::ops::Mul;

#[test]
pub fn handle_configure() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    let msg = HandleMsg::Configure {
        manager: Option::from(HumanAddr::from(TEST_NON_MANAGER)),
        refund_denom: Option::from(TEST_OWNER.to_string()),
        base_price: Option::from(Decimal256::from_str("10.10").unwrap()),
    };

    let non_manager = mock_env(TEST_NON_MANAGER, &[]);
    let _res = contract::handle(&mut deps, non_manager, msg.clone())
        .expect_err("should fail if non-manager called this method");

    let manager = mock_env(TEST_MANAGER, &[]);
    let _res = contract::handle(&mut deps, manager, msg).unwrap();

    let config = config::read(&deps.storage).unwrap();

    assert_eq!(
        config,
        config::Config {
            manager: HumanAddr::from(TEST_NON_MANAGER),
            refund_denom: TEST_OWNER.to_string(),
            base_price: Decimal256::from_str("10.10").unwrap(),
        }
    )
}

#[test]
pub fn handle_refund() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    let buyers: Vec<Buyer> = (0..10)
        .map(|x| Buyer {
            address: HumanAddr::from(format!("TEST_{}", x)),
            amount: Uint256::from(10u64.pow(6).mul(x)),
        })
        .collect();
    for buyer in &buyers {
        user::store(
            &mut deps.storage,
            &deps.api.canonical_address(&buyer.address).unwrap(),
            &user::User {
                amount: buyer.amount,
            },
        )
        .unwrap();
    }

    let manager = mock_env(TEST_MANAGER, &[]);

    let msg = HandleMsg::Refund {
        start_after: None,
        limit: None,
    };
    let res = contract::handle(&mut deps, manager, msg).unwrap();
    assert_eq!(res.messages.len(), 9); // except 0

    for buyer in &buyers {
        let user_state = user_state::read(
            &deps.storage,
            &deps.api.canonical_address(&buyer.address).unwrap(),
        )
        .unwrap();
        assert_eq!(user_state.processed, true);
    }
}
