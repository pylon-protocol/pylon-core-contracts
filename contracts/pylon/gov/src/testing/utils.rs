use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::*;
use pylon_token::gov::{HandleMsg, InitMsg};

use crate::contract;
use crate::testing::constants::*;
use crate::testing::mock_querier::WasmMockQuerier;

pub fn mock_init(mut deps: &mut Extern<MockStorage, MockApi, WasmMockQuerier>) {
    let msg = InitMsg {
        quorum: Decimal::percent(DEFAULT_QUORUM),
        threshold: Decimal::percent(DEFAULT_THRESHOLD),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        expiration_period: DEFAULT_EXPIRATION_PERIOD,
        proposal_deposit: Uint128(DEFAULT_PROPOSAL_DEPOSIT),
        snapshot_period: DEFAULT_FIX_PERIOD,
    };

    let env = mock_env(TEST_CREATOR, &[]);
    let _res =
        contract::init(&mut deps, env.clone(), msg).expect("contract successfully handles InitMsg");

    let msg = HandleMsg::RegisterContracts {
        pylon_token: HumanAddr::from(VOTING_TOKEN),
    };
    let _res = contract::handle(&mut deps, env, msg)
        .expect("contract successfully handles RegisterContracts");
}

pub fn mock_env_height(sender: &str, sent: &[Coin], height: u64, time: u64) -> Env {
    let mut env = mock_env(sender, sent);
    env.block.height = height;
    env.block.time = time;
    env
}

pub fn init_msg() -> InitMsg {
    InitMsg {
        quorum: Decimal::percent(DEFAULT_QUORUM),
        threshold: Decimal::percent(DEFAULT_THRESHOLD),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        expiration_period: DEFAULT_EXPIRATION_PERIOD,
        proposal_deposit: Uint128(DEFAULT_PROPOSAL_DEPOSIT),
        snapshot_period: DEFAULT_FIX_PERIOD,
    }
}
