use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{coins, Api, Decimal, StdError, Uint128};
use pylon_token::gov_msg::InstantiateMsg;

use crate::contract;
use crate::error::ContractError;
use crate::state::config::{config_r, Config};
use crate::state::state::{state_r, State};
use crate::testing::constants::*;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils::instantiate_msg;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &coins(2, VOTING_TOKEN));
    let res = contract::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config = config_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        Config {
            pylon_token: deps.api.addr_canonicalize(VOTING_TOKEN).unwrap(),
            owner: deps.api.addr_canonicalize(TEST_CREATOR).unwrap(),
            quorum: Decimal::percent(DEFAULT_QUORUM),
            threshold: Decimal::percent(DEFAULT_THRESHOLD),
            voting_period: DEFAULT_VOTING_PERIOD,
            timelock_period: DEFAULT_TIMELOCK_PERIOD,
            expiration_period: 0u64, // Deprecated
            proposal_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
            snapshot_period: DEFAULT_FIX_PERIOD
        }
    );

    let state = state_r(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 0,
            total_share: Uint128::zero(),
            total_deposit: Uint128::zero(),
        }
    );
}

#[test]
fn fails_init_invalid_quorum() {
    let mut deps = mock_dependencies(&[]);
    let info = mock_info("voter", &coins(11, VOTING_TOKEN));
    let msg = InstantiateMsg {
        voting_token: VOTING_TOKEN.to_string(),
        quorum: Decimal::percent(101),
        threshold: Decimal::percent(DEFAULT_THRESHOLD),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        proposal_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        snapshot_period: DEFAULT_FIX_PERIOD,
    };

    let res = contract::instantiate(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "quorum must be 0 to 1")
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fails_init_invalid_threshold() {
    let mut deps = mock_dependencies(&[]);
    let info = mock_info("voter", &coins(11, VOTING_TOKEN));
    let msg = InstantiateMsg {
        voting_token: VOTING_TOKEN.to_string(),
        quorum: Decimal::percent(DEFAULT_QUORUM),
        threshold: Decimal::percent(101),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        proposal_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        snapshot_period: DEFAULT_FIX_PERIOD,
    };

    let res = contract::instantiate(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Std(StdError::GenericErr { msg, .. })) => {
            assert_eq!(msg, "threshold must be 0 to 1")
        }
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
