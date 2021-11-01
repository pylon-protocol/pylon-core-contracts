use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{Decimal, DepsMut, Env, Timestamp, Uint128};
use pylon_token::gov::InstantiateMsg;

use crate::contract;
use crate::testing::constants::*;

pub fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        voting_token: VOTING_TOKEN.to_string(),
        quorum: Decimal::percent(DEFAULT_QUORUM),
        threshold: Decimal::percent(DEFAULT_THRESHOLD),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        proposal_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        snapshot_period: DEFAULT_FIX_PERIOD,
    }
}

pub fn mock_env_height(height: u64, time: u64) -> Env {
    let mut env = mock_env();
    env.block.height = height;
    env.block.time = Timestamp::from_seconds(time);
    env
}

pub fn mock_instantiate(deps: DepsMut) {
    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = contract::instantiate(deps, mock_env(), info, msg)
        .expect("testing: contract successfully handles InstantiateMsg");
}
