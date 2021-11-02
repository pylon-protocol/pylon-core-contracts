use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{from_binary, Decimal, Uint128};
use pylon_token::gov_msg::{ExecuteMsg, QueryMsg};
use pylon_token::gov_resp::ConfigResponse;

use crate::contract;
use crate::error::ContractError;
use crate::testing::constants::*;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils::mock_instantiate;

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // update owner
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("addr0001".to_string()),
        quorum: None,
        threshold: None,
        voting_period: None,
        timelock_period: None,
        proposal_deposit: None,
        snapshot_period: None,
    };

    let res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = contract::query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("addr0001", config.owner.as_str());
    assert_eq!(Decimal::percent(DEFAULT_QUORUM), config.quorum);
    assert_eq!(Decimal::percent(DEFAULT_THRESHOLD), config.threshold);
    assert_eq!(DEFAULT_VOTING_PERIOD, config.voting_period);
    assert_eq!(DEFAULT_TIMELOCK_PERIOD, config.timelock_period);
    assert_eq!(DEFAULT_PROPOSAL_DEPOSIT, config.proposal_deposit.u128());

    // update left items
    let info = mock_info("addr0001", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        quorum: Some(Decimal::percent(20)),
        threshold: Some(Decimal::percent(75)),
        voting_period: Some(20000u64),
        timelock_period: Some(20000u64),
        proposal_deposit: Some(Uint128::from(123u128)),
        snapshot_period: Some(11),
    };

    let res = contract::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = contract::query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("addr0001", config.owner.as_str());
    assert_eq!(Decimal::percent(20), config.quorum);
    assert_eq!(Decimal::percent(75), config.threshold);
    assert_eq!(20000u64, config.voting_period);
    assert_eq!(20000u64, config.timelock_period);
    assert_eq!(123u128, config.proposal_deposit.u128());
    assert_eq!(11u64, config.snapshot_period);

    // Unauthorzied err
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        quorum: None,
        threshold: None,
        voting_period: None,
        timelock_period: None,
        proposal_deposit: None,
        snapshot_period: None,
    };

    let res = contract::execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}
