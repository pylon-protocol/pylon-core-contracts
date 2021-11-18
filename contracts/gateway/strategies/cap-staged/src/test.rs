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
const USER: &str = "user";

fn init_contract(
    deps: &mut OwnedDeps<MockStorage, MockApi, CustomMockWasmQuerier>,
    gov: String,
    stages: Vec<state::Stage>,
) -> (Env, MessageInfo) {
    let env = mock_env();
    let info = mock_info(OWNER, &[]);

    let resp = contract::instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        contract::InstantiateMsg { gov, stages },
    )
    .expect("testing: should init contract");
    assert_eq!(resp, Response::default());

    (env, info)
}

#[test]
fn instantiate() {
    let mut deps = mock_dependencies(&[]);

    let stages: Vec<state::Stage> = vec![state::Stage::default(), state::Stage::default()];
    let _ = init_contract(&mut deps, GOV.to_string(), stages.clone());

    let config = state::config_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        state::Config {
            owner: OWNER.to_string(),
            gov: GOV.to_string(),
            stages
        }
    );
}

#[test]
fn execute_configure() {
    let mut deps = mock_dependencies(&[]);

    let mut stages: Vec<state::Stage> = vec![state::Stage::default(), state::Stage::default()];
    let (env, owner) = init_contract(&mut deps, GOV.to_string(), stages.clone());

    stages.push(state::Stage::default());
    let msg = ExecuteMsg::Configure {
        owner: Option::from(NEW_OWNER.to_string()),
        gov: Option::from(NEW_GOV.to_string()),
        stages: Option::from(stages.clone()),
    };

    contract::execute(
        deps.as_mut(),
        env.clone(),
        mock_info(USER, &[]),
        msg.clone(),
    )
    .expect_err("testing: should not able to configure settings (non-owner)");
    let resp = contract::execute(deps.as_mut(), env, owner, msg)
        .expect("testing: should able to configure settings (owner)");
    assert_eq!(resp, Response::default());

    let config = state::config_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        state::Config {
            owner: NEW_OWNER.to_string(),
            gov: NEW_GOV.to_string(),
            stages
        }
    );
}

#[test]
fn query_available_cap() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.register_wasm_smart_query_handler(
        GOV.to_string(),
        Box::new(|x| match from_binary::<GovQueryMsg>(x).unwrap() {
            GovQueryMsg::Staker { address } => to_binary(&StakerResponse {
                balance: match address.as_str() {
                    "tester_1" => Uint128::from(5u64),
                    "tester_2" => Uint128::from(15u64),
                    _ => Uint128::zero(),
                },
                share: Default::default(),
                claimable_airdrop: vec![],
                locked_balance: vec![],
            }),
            _ => Err(StdError::generic_err("not implemented")),
        }),
    );

    let stages: Vec<state::Stage> = vec![
        state::Stage {
            // 0 <= x < 10
            from: Uint256::from(0u64),
            to: None,
            min_cap: Uint256::from(0u64),
            max_cap: Uint256::from(100u64),
        },
        state::Stage {
            // 10 <= x < 20
            from: Uint256::from(10u64),
            to: Option::from(Uint256::from(20u64)),
            min_cap: Uint256::from(100u64),
            max_cap: Uint256::from(200u64),
        },
    ];

    init_contract(&mut deps, GOV.to_string(), stages);

    let resp: AvailableCapOfResponse = from_binary(
        &contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AvailableCapOf {
                address: "tester_1".to_string(),
                amount: Uint256::from(0u64),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(resp.amount, Option::from(Uint256::from(100u64)));

    let resp: AvailableCapOfResponse = from_binary(
        &contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AvailableCapOf {
                address: "tester_2".to_string(),
                amount: Uint256::from(0u64),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(resp.amount, Option::from(Uint256::from(200u64)));

    let resp: AvailableCapOfResponse = from_binary(
        &contract::query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::AvailableCapOf {
                address: "tester_2".to_string(),
                amount: Uint256::from(100u64),
            },
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(resp.amount, Option::from(Uint256::from(100u64)));
}
