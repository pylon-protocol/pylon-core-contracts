use cosmwasm_bignumber::Uint256;
use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{from_binary, Coin, Decimal, Uint128};
use pylon_core::pool_v2_msg::QueryMsg;
use pylon_core::pool_v2_resp::{
    ClaimableRewardResponse, ConfigResponse, DepositAmountResponse, TotalDepositAmountResponse,
};
use pylon_core::test_constant::*;
use pylon_utils::mock_token::MockToken;
use pylon_utils::tax::deduct_tax;
use std::ops::Mul;

use crate::contract;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;
use pylon_utils::mock_tax::MockTax;

#[test]
fn query_config() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);

    let msg = QueryMsg::Config {};
    let bin_res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
    let res: ConfigResponse = from_binary(&bin_res).unwrap();
    assert_eq!(
        res,
        ConfigResponse {
            id: TEST_POOL_ID,
            name: TEST_POOL_NAME.to_string(),
            factory: TEST_FACTORY.to_string(),
            beneficiary: TEST_BENEFICIARY.to_string(),
            yield_adapter: TEST_ADAPTER.to_string(),
            input_denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
            yield_token: TEST_TOKEN_YIELD.to_string(),
            dp_token: TEST_TOKEN_POOL.to_string()
        }
    )
}

#[test]
fn query_deposit_amount() {
    let mut deps = mock_dependencies(&[]);
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
        owner: TEST_USER.to_string(),
    };
    let res: DepositAmountResponse =
        from_binary(&contract::query(deps.as_ref(), mock_env(), msg).unwrap()).unwrap();
    assert_eq!(
        res,
        DepositAmountResponse {
            amount: Uint256::from(TEST_TOKEN_POOL_SUPPLY)
        }
    )
}

#[test]
fn query_total_deposit_amount() {
    let mut deps = mock_dependencies(&[]);
    let _ = utils::initialize(&mut deps);

    let msg = QueryMsg::TotalDepositAmount {};
    let bin_res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
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
    let mut deps = mock_dependencies(&[]);
    deps.querier.with_tax(MockTax::new(
        Decimal::percent(1),
        &[(
            &TEST_ADAPTER_INPUT_DENOM.to_string(),
            &Uint128::from(1000000u128),
        )],
    ));
    deps.querier.token.with_balances(&[
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

    let _ = utils::initialize(&mut deps);

    let msg = QueryMsg::ClaimableReward {};
    let bin_res = contract::query(deps.as_ref(), mock_env(), msg).unwrap();
    let res: ClaimableRewardResponse = from_binary(&bin_res).unwrap();
    let reward = deduct_tax(
        deps.as_ref(),
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
