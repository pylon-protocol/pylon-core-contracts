use crate::contract;
use crate::testing::constants::{
    TEST_ADAPTER, TEST_ADAPTER_INPUT_DENOM, TEST_BENEFICIARY, TEST_FACTORY, TEST_FACTORY_FEE_RATE,
    TEST_POOL_ID, TEST_POOL_NAME, TEST_TOKEN_POOL, TEST_TOKEN_POOL_SUPPLY, TEST_TOKEN_YIELD,
    TEST_TOKEN_YIELD_SUPPLY, TEST_USER,
};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::mock_tax::MockTax;
use crate::testing::mock_token::{balances_to_map, MockToken};
use crate::testing::utils;
use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::MOCK_CONTRACT_ADDR;
use cosmwasm_std::{from_binary, Coin, Decimal, HumanAddr, Uint128};
use pylon_core::pool_msg::QueryMsg;
use pylon_core::pool_resp::{
    ClaimableRewardResponse, ConfigResponse, DepositAmountResponse, TotalDepositAmountResponse,
};
use pylon_utils::tax::deduct_tax;
use std::ops::Mul;

#[test]
fn query_config() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    let msg = QueryMsg::Config {};
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: ConfigResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        ConfigResponse {
            id: TEST_POOL_ID,
            name: TEST_POOL_NAME.to_string(),
            factory: HumanAddr::from(TEST_FACTORY),
            beneficiary: HumanAddr::from(TEST_BENEFICIARY),
            yield_adapter: HumanAddr::from(TEST_ADAPTER),
            input_denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
            yield_token: HumanAddr::from(TEST_TOKEN_YIELD),
            dp_token: HumanAddr::from(TEST_TOKEN_POOL)
        }
    )
}

#[test]
fn query_deposit_amount() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    deps.querier.with_token(MockToken::new(
        &[],
        &[(
            &TEST_TOKEN_POOL.to_string(),
            &[(
                &TEST_USER.to_string(),
                &Uint128::from(TEST_TOKEN_POOL_SUPPLY),
            )],
        )],
    ));

    let msg = QueryMsg::DepositAmountOf {
        owner: HumanAddr::from(TEST_USER),
    };
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: DepositAmountResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        DepositAmountResponse {
            amount: Uint256::from(TEST_TOKEN_POOL_SUPPLY)
        }
    )
}

#[test]
fn query_total_deposit_amount() {
    let mut deps = mock_dependencies(20, &[]);
    let _ = utils::initialize(&mut deps);

    let msg = QueryMsg::TotalDepositAmount {};
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: TotalDepositAmountResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        TotalDepositAmountResponse {
            amount: Uint256::from(TEST_TOKEN_POOL_SUPPLY)
        }
    )
}

#[test]
fn query_claimable_reward() {
    let mut deps = mock_dependencies(20, &[]);
    deps.querier.with_tax(MockTax::new(
        Decimal::percent(1),
        &[(
            &TEST_ADAPTER_INPUT_DENOM.to_string(),
            &Uint128::from(1000000u128),
        )],
    ));

    let mut mock_token = MockToken::default();
    mock_token.balances = balances_to_map(&[
        (
            &TEST_TOKEN_YIELD.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(TEST_TOKEN_YIELD_SUPPLY),
            )],
        ),
        (
            &TEST_TOKEN_POOL.to_string(),
            &[(
                &MOCK_CONTRACT_ADDR.to_string(),
                &Uint128::from(TEST_TOKEN_POOL_SUPPLY),
            )],
        ),
    ]);
    deps.querier.with_token(mock_token);

    let _ = utils::initialize(&mut deps);

    let msg = QueryMsg::ClaimableReward {};
    let bin_res = contract::query(&deps, msg).unwrap();
    let res: ClaimableRewardResponse = from_binary(&bin_res).unwrap();
    let reward = deduct_tax(
        &deps,
        Coin {
            denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
            amount: Uint128::from(TEST_TOKEN_POOL_SUPPLY),
        },
    )
    .unwrap();
    assert_eq!(
        res,
        ClaimableRewardResponse {
            amount: Uint256::from(
                reward
                    .amount
                    .mul(Decimal::percent(100 - TEST_FACTORY_FEE_RATE))
            ),
            fee: Uint256::from(reward.amount.mul(Decimal::percent(TEST_FACTORY_FEE_RATE)))
        }
    )
}
