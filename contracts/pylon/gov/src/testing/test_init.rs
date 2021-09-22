use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{coins, Api, CanonicalAddr, Decimal, HumanAddr, Uint128};
use pylon_token::gov::HandleMsg;

use crate::contract;
use crate::state::{config, state};
use crate::testing::constants::*;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = utils::init_msg();
    let env = mock_env(TEST_CREATOR, &coins(2, VOTING_TOKEN));
    let res = contract::init(&mut deps, env.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config = config::read(&deps.storage).load().unwrap();
    assert_eq!(
        config,
        config::Config {
            pylon_token: CanonicalAddr::default(),
            owner: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_CREATOR))
                .unwrap(),
            quorum: Decimal::percent(DEFAULT_QUORUM),
            threshold: Decimal::percent(DEFAULT_THRESHOLD),
            voting_period: DEFAULT_VOTING_PERIOD,
            timelock_period: DEFAULT_TIMELOCK_PERIOD,
            expiration_period: DEFAULT_EXPIRATION_PERIOD,
            proposal_deposit: Uint128(DEFAULT_PROPOSAL_DEPOSIT),
            snapshot_period: DEFAULT_FIX_PERIOD
        }
    );

    let msg = HandleMsg::RegisterContracts {
        pylon_token: HumanAddr::from(VOTING_TOKEN),
    };
    let _res = contract::handle(&mut deps, env, msg).unwrap();
    let config = config::read(&deps.storage).load().unwrap();
    assert_eq!(
        config.pylon_token,
        deps.api
            .canonical_address(&HumanAddr::from(VOTING_TOKEN))
            .unwrap()
    );

    let state = state::read(&deps.storage).load().unwrap();
    assert_eq!(
        state,
        state::State {
            contract_addr: deps
                .api
                .canonical_address(&HumanAddr::from(MOCK_CONTRACT_ADDR))
                .unwrap(),
            poll_count: 0,
            total_share: Uint128::zero(),
            total_deposit: Uint128::zero(),
        }
    );
}
