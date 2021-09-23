use cosmwasm_std::testing::{mock_env, MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use cosmwasm_std::*;
use cw20::Cw20ReceiveMsg;
use pylon_token::common::OrderBy;
use pylon_token::gov::{Cw20HookMsg, HandleMsg, QueryMsg, StakerResponse, StakersResponse};

use crate::contract;
use crate::testing::constants::*;
use crate::testing::mock_querier::{mock_dependencies, WasmMockQuerier};
use crate::testing::utils;

#[test]
fn query_stakers() {
    let mut deps = mock_dependencies(20, &[]);
    utils::mock_init(&mut deps);

    let stake_amount = Uint128::from(100000u64);

    let update_contract_balance = |deps: &mut Extern<MockStorage, MockApi, WasmMockQuerier>,
                                   address: &str,
                                   balance: &Uint128| {
        deps.querier.with_token_balances(&[(
            &HumanAddr::from(VOTING_TOKEN),
            &[
                (&HumanAddr::from(TEST_VOTER), &stake_amount),
                (&HumanAddr::from(TEST_VOTER_2), &stake_amount),
                (&HumanAddr::from(TEST_VOTER_3), &stake_amount),
                (&HumanAddr::from(address), balance),
            ],
        )]);
    };

    let converter = |v: &str| -> (HumanAddr, StakerResponse) {
        (
            HumanAddr::from(v),
            StakerResponse {
                balance: stake_amount,
                share: stake_amount,
                locked_balance: vec![],
            },
        )
    };

    let stake =
        |deps: &mut Extern<MockStorage, MockApi, WasmMockQuerier>, v: &str| -> HandleResponse {
            let msg = HandleMsg::Receive(Cw20ReceiveMsg {
                sender: HumanAddr::from(v),
                amount: stake_amount,
                msg: Option::from(to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap()),
            });
            let res = contract::handle(deps, mock_env(VOTING_TOKEN, &[]), msg)
                .expect("testing: must stake successfully");
            assert_eq!(res.data, None);
            assert_eq!(res.messages, vec![]);

            res
        };

    /* =========================== 1 =========================== */
    update_contract_balance(&mut deps, MOCK_CONTRACT_ADDR, &stake_amount);

    let _ = stake(&mut deps, TEST_VOTER);
    let query = QueryMsg::Stakers {
        start_after: None,
        limit: None,
        order: None,
    };
    let raw_res = contract::query(&deps, query).expect("testing: must success to query stakers");
    let res: StakersResponse = from_binary(&raw_res).unwrap();
    assert_eq!(res.stakers, [TEST_VOTER].map(converter));

    /* =========================== 2 =========================== */
    update_contract_balance(
        &mut deps,
        MOCK_CONTRACT_ADDR,
        &Uint128::from(stake_amount.u128() * 2),
    );

    let _ = stake(&mut deps, TEST_VOTER_2);
    let query = QueryMsg::Stakers {
        start_after: None,
        limit: None,
        order: None,
    };
    let raw_res = contract::query(&deps, query).expect("testing: must success to query stakers");
    let res: StakersResponse = from_binary(&raw_res).unwrap();
    assert_eq!(res.stakers, [TEST_VOTER, TEST_VOTER_2].map(converter));

    /* =========================== 3 =========================== */
    update_contract_balance(
        &mut deps,
        MOCK_CONTRACT_ADDR,
        &Uint128::from(stake_amount.u128() * 3),
    );

    let _ = stake(&mut deps, TEST_VOTER_3);
    let query = QueryMsg::Stakers {
        start_after: None,
        limit: None,
        order: None,
    };
    let raw_res = contract::query(&deps, query).expect("testing: must success to query stakers");
    let res: StakersResponse = from_binary(&raw_res).unwrap();
    assert_eq!(
        res.stakers,
        [TEST_VOTER, TEST_VOTER_2, TEST_VOTER_3].map(converter)
    );

    /* =========================== start_after =========================== */
    let query = QueryMsg::Stakers {
        start_after: Option::from(HumanAddr::from(TEST_VOTER)),
        limit: None,
        order: None,
    };
    let raw_res = contract::query(&deps, query).expect("testing: must success to query stakers");
    let res: StakersResponse = from_binary(&raw_res).unwrap();
    assert_eq!(res.stakers, [TEST_VOTER_2, TEST_VOTER_3].map(converter));

    /* =========================== limit =========================== */
    let query = QueryMsg::Stakers {
        start_after: None,
        limit: Option::from(1),
        order: None,
    };
    let raw_res = contract::query(&deps, query).expect("testing: must success to query stakers");
    let res: StakersResponse = from_binary(&raw_res).unwrap();
    assert_eq!(res.stakers, [TEST_VOTER].map(converter));

    /* =========================== order =========================== */
    let query = QueryMsg::Stakers {
        start_after: None,
        limit: None,
        order: Option::from(OrderBy::Desc),
    };
    let raw_res = contract::query(&deps, query).expect("testing: must success to query stakers");
    let res: StakersResponse = from_binary(&raw_res).unwrap();
    assert_eq!(
        res.stakers,
        [TEST_VOTER_3, TEST_VOTER_2, TEST_VOTER].map(converter)
    );
}
