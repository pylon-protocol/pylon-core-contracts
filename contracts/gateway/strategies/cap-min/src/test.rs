use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockStorage};
use cosmwasm_std::{
    from_binary, to_binary, Env, MessageInfo, OwnedDeps, Response, StdError, Uint128,
};
use pylon_gateway::cap_strategy_msg::QueryMsg;
use pylon_gateway::cap_strategy_resp::AvailableCapOfResponse;
use pylon_token::gov_msg::QueryMsg as GovQueryMsg;
use pylon_token::gov_resp::StakerResponse;

use crate::contract::ExecuteMsg;
use crate::mock_querier::{mock_dependencies, CustomMockWasmQuerier};
use crate::{contract, state};

const GOV: &str = "gov";
const NEW_GOV: &str = "new_gov";
const OWNER: &str = "owner";
const NEW_OWNER: &str = "new_owner";

fn init_contract(
    deps: &mut OwnedDeps<MockStorage, MockApi, CustomMockWasmQuerier>,
    gov: String,
    minimum_stake_amount: Uint256,
) -> (Env, MessageInfo) {
    let env = mock_env();
    let info = mock_info(OWNER, &[]);

    let resp = contract::instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        contract::InstantiateMsg {
            gov,
            minimum_stake_amount,
        },
    )
    .expect("testing: should init contract");
    assert_eq!(resp, Response::default());

    (env, info)
}

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);

    let minimum_stake_amount = Uint256::from(1000u64);
    let _ = init_contract(&mut deps, GOV.to_string(), minimum_stake_amount);

    let config = state::config_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        state::Config {
            owner: OWNER.to_string(),
            gov: GOV.to_string(),
            minimum_stake_amount
        }
    );
}

#[test]
fn execute_configure() {
    let mut deps = mock_dependencies(&[]);

    let minimum_stake_amount = Uint256::from(1000u64);
    let (env, owner) = init_contract(&mut deps, GOV.to_string(), minimum_stake_amount);

    let msg = ExecuteMsg::Configure {
        owner: Option::from(NEW_OWNER.to_string()),
        gov: Option::from(NEW_GOV.to_string()),
        minimum_stake_amount: Option::from(minimum_stake_amount),
    };
    let resp = contract::execute(deps.as_mut(), env, owner, msg)
        .expect("testing: should able to configure settings");
    assert_eq!(resp, Response::default());

    let config = state::config_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        state::Config {
            owner: NEW_OWNER.to_string(),
            gov: NEW_GOV.to_string(),
            minimum_stake_amount
        }
    );
}

#[test]
fn query_available_cap() {
    let minimum_stake_amount = Uint256::from(1000u64);

    let mut deps = mock_dependencies(&[]);
    deps.querier.register_wasm_smart_query_handler(
        GOV.to_string(),
        Box::new(|x| match from_binary::<GovQueryMsg>(x).unwrap() {
            GovQueryMsg::Staker { address } => to_binary(&StakerResponse {
                balance: match address.as_str() {
                    "tester_1" => Uint128::from(999u64),
                    "tester_2" => Uint128::from(1000u64),
                    _ => Uint128::zero(),
                },
                share: Default::default(),
                claimable_airdrop: vec![],
                locked_balance: vec![],
            }),
            _ => Err(StdError::generic_err("not implemented")),
        }),
    );

    let (env, _) = init_contract(&mut deps, GOV.to_string(), minimum_stake_amount);

    let amount = Uint256::from(10u64);

    // lt minimum_stake_amount
    let msg = QueryMsg::AvailableCapOf {
        address: "tester_1".to_string(),
        amount,
    };
    let resp = from_binary::<AvailableCapOfResponse>(
        &contract::query(deps.as_ref(), env.clone(), msg)
            .expect("testing: should able to query available cap"),
    )
    .unwrap();
    assert_eq!(resp.amount, Option::Some(Uint256::from(0u64)));
    assert!(!resp.unlimited);

    // gt minimum_stake_amount
    let msg = QueryMsg::AvailableCapOf {
        address: "tester_2".to_string(),
        amount,
    };
    let resp = from_binary::<AvailableCapOfResponse>(
        &contract::query(deps.as_ref(), env, msg)
            .expect("testing: should able to query available cap"),
    )
    .unwrap();
    assert_eq!(resp.amount, Option::None);
    assert!(resp.unlimited);
}
