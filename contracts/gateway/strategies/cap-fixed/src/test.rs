use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
};
use cosmwasm_std::{from_binary, Env, MessageInfo, OwnedDeps, Response};
use pylon_gateway::cap_strategy_msg::QueryMsg;
use pylon_gateway::cap_strategy_resp::AvailableCapOfResponse;

use crate::contract::ExecuteMsg;
use crate::{contract, state};

const OWNER: &str = "owner";
const NEW_OWNER: &str = "new_owner";

fn init_contract(
    deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
    min_user_cap: Uint256,
    max_user_cap: Uint256,
) -> (Env, MessageInfo) {
    let env = mock_env();
    let info = mock_info(OWNER, &[]);

    let resp = contract::instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        contract::InstantiateMsg {
            min_user_cap,
            max_user_cap,
        },
    )
    .expect("testing: should init contract");
    assert_eq!(resp, Response::default());

    (env, info)
}

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);

    let min_user_cap = Uint256::from(10u64);
    let max_user_cap = Uint256::from(1500u64);
    let _ = init_contract(&mut deps, min_user_cap, max_user_cap);

    let config = state::config_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        state::Config {
            owner: OWNER.to_string(),
            min_user_cap,
            max_user_cap
        }
    );
}

#[test]
fn execute_configure() {
    let mut deps = mock_dependencies(&[]);

    let mut min_user_cap = Uint256::from(10u64);
    let mut max_user_cap = Uint256::from(1500u64);
    let (env, owner) = init_contract(&mut deps, min_user_cap, max_user_cap);

    min_user_cap += Uint256::from(100u64);
    max_user_cap = max_user_cap - Uint256::from(100u64);

    let msg = ExecuteMsg::Configure {
        owner: Option::from(NEW_OWNER.to_string()),
        min_user_cap: Option::from(min_user_cap),
        max_user_cap: Option::from(max_user_cap),
    };
    let resp = contract::execute(deps.as_mut(), env, owner, msg)
        .expect("testing: should able to configure settings");
    assert_eq!(resp, Response::default());

    let config = state::config_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        state::Config {
            owner: NEW_OWNER.to_string(),
            min_user_cap,
            max_user_cap
        }
    );
}

#[test]
fn query_available_cap() {
    let mut deps = mock_dependencies(&[]);

    let min_user_cap = Uint256::from(10u64);
    let max_user_cap = Uint256::from(1500u64);
    let (env, _) = init_contract(&mut deps, min_user_cap, max_user_cap);

    // lt min_user_cap
    let amount = min_user_cap - Uint256::from(1u64);
    let msg = QueryMsg::AvailableCapOf {
        address: "".to_string(),
        amount,
    };
    let resp = from_binary::<AvailableCapOfResponse>(
        &contract::query(deps.as_ref(), env.clone(), msg)
            .expect("testing: should able to query available cap"),
    )
    .unwrap();
    assert_eq!(resp.amount, Option::from(max_user_cap - amount));

    // gt min_user_cap && lt max_user_cap
    let amount = min_user_cap + Uint256::from(1u64);
    let msg = QueryMsg::AvailableCapOf {
        address: "".to_string(),
        amount,
    };
    let resp = from_binary::<AvailableCapOfResponse>(
        &contract::query(deps.as_ref(), env.clone(), msg)
            .expect("testing: should able to query available cap"),
    )
    .unwrap();
    assert_eq!(resp.amount, Option::from(max_user_cap - amount));

    let amount = max_user_cap - Uint256::from(1u64);
    let msg = QueryMsg::AvailableCapOf {
        address: "".to_string(),
        amount,
    };
    let resp = from_binary::<AvailableCapOfResponse>(
        &contract::query(deps.as_ref(), env.clone(), msg)
            .expect("testing: should able to query available cap"),
    )
    .unwrap();
    assert_eq!(resp.amount, Option::from(max_user_cap - amount));

    // gt max_user_cap
    let amount = max_user_cap + Uint256::from(1u64);
    let msg = QueryMsg::AvailableCapOf {
        address: "".to_string(),
        amount,
    };
    let resp = from_binary::<AvailableCapOfResponse>(
        &contract::query(deps.as_ref(), env, msg)
            .expect("testing: should able to query available cap"),
    )
    .unwrap();
    assert_eq!(resp.amount, Option::from(Uint256::zero()));
}
