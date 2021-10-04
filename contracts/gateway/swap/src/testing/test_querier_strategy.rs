use crate::querier::strategy;
use crate::state::{config, user};
use crate::testing::constants::TEST_USER_1;
use crate::testing::mock_querier::mock_dependencies;
use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::Api;
use pylon_gateway::swap_msg::Strategy;
use std::str::FromStr;

#[test]
fn strategy_claimable_token_of() {
    let mut deps = mock_dependencies(&[]);

    config::store(deps.as_mut().storage)
        .save(&config::Config {
            owner: "".to_string(),
            beneficiary: "".to_string(),
            price: Default::default(),
            start: 0,
            finish: 0,
            cap_strategy: None,
            distribution_strategy: vec![
                Strategy::Lockup {
                    release_time: 1,
                    release_amount: Decimal256::from_str("0.25").unwrap(),
                },
                Strategy::Lockup {
                    release_time: 2,
                    release_amount: Decimal256::from_str("0.75").unwrap(),
                },
            ],
            whitelist_enabled: false,
            swap_pool_size: Default::default(),
        })
        .unwrap();

    let user_addr = &deps.api.addr_canonicalize(TEST_USER_1).unwrap();
    user::store(
        deps.as_mut().storage,
        user_addr,
        &user::User {
            whitelisted: false,
            swapped_in: Default::default(),
            swapped_out: Uint256::from(100u64),
            swapped_out_claimed: Default::default(),
        },
    )
    .unwrap();

    assert_eq!(
        strategy::claimable_token_of(deps.as_ref(), 0, TEST_USER_1.to_string()).unwrap(),
        Uint256::from(0u64),
    );
    assert_eq!(
        strategy::claimable_token_of(deps.as_ref(), 1, TEST_USER_1.to_string()).unwrap(),
        Uint256::from(25u64),
    );
    assert_eq!(
        strategy::claimable_token_of(deps.as_ref(), 2, TEST_USER_1.to_string()).unwrap(),
        Uint256::from(100u64),
    );
}
