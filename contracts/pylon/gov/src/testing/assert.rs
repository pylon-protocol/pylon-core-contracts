use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_std::{attr, Deps, Response, Uint128};
use pylon_token::gov_msg::VoteOption;

use crate::state::state::{state_r, State};
use crate::testing::constants::*;

// helper to confirm the expected create_poll response
pub fn assert_create_poll_result(
    poll_id: u64,
    end_height: u64,
    creator: &str,
    execute_res: Response,
    deps: Deps,
) {
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "create_poll"),
            attr("creator", creator),
            attr("poll_id", poll_id.to_string()),
            attr("end_height", end_height.to_string()),
        ]
    );

    //confirm poll count
    let state = state_r(deps.storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 1,
            total_share: Uint128::zero(),
            total_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        }
    );
}

pub fn assert_stake_tokens_result(
    total_share: u128,
    total_deposit: u128,
    new_share: u128,
    poll_count: u64,
    execute_res: Response,
    deps: Deps,
) {
    assert_eq!(
        execute_res.attributes.get(2).expect("no log"),
        &attr("share", new_share.to_string())
    );

    let state = state_r(deps.storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count,
            total_share: Uint128::from(total_share),
            total_deposit: Uint128::from(total_deposit),
        }
    );
}

pub fn assert_cast_vote_success(
    voter: &str,
    amount: u128,
    poll_id: u64,
    vote_option: VoteOption,
    execute_res: Response,
) {
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", poll_id.to_string()),
            attr("amount", amount.to_string()),
            attr("voter", voter),
            attr("vote_option", vote_option.to_string()),
        ]
    );
}
