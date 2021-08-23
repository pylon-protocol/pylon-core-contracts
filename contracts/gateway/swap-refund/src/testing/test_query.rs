use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::{from_binary, Api, HumanAddr};
use std::ops::Mul;

use crate::contract;
use crate::contract::QueryMsg;
use crate::handler::query::BuyersResponse;
use crate::state::user;
use crate::testing::utils;

#[test]
pub fn query_buyers() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    for x in 0..10u64 {
        let address = HumanAddr::from(format!("TEST_{}", x));
        user::store(
            &mut deps.storage,
            &deps.api.canonical_address(&address).unwrap(),
            &user::User {
                amount: Uint256::from(10u64.pow(6).mul(x)),
            },
        )
        .unwrap(); // must
    }

    let msg = QueryMsg::Buyers {
        start_after: None,
        limit: None,
    };
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: BuyersResponse = from_binary(&bin_res).unwrap();
    assert_eq!(res.buyers.len(), 10);

    let msg = QueryMsg::Buyers {
        start_after: Option::from(HumanAddr::from("TEST_4")),
        limit: None,
    };
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: BuyersResponse = from_binary(&bin_res).unwrap();
    assert_eq!(res.buyers.len(), 5);
}
