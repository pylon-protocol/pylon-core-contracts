use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    from_binary, to_binary, BankMsg, Coin, CosmosMsg, Decimal, HumanAddr, Uint128, WasmMsg,
};
use cw20::{Cw20HandleMsg, TokenInfoResponse};
use pylon_core::pool_msg::{HandleMsg, QueryMsg};
use pylon_core::pool_resp::ClaimableRewardResponse;

use crate::contract;
use crate::testing::constants::{
    TEST_ADAPTER, TEST_ADAPTER_EXCHANGE_RATE, TEST_ADAPTER_INPUT_DENOM, TEST_ADAPTER_TARGET,
    TEST_BENEFICIARY, TEST_FACTORY, TEST_FACTORY_FEE_COLLECTOR, TEST_FACTORY_FEE_RATE,
    TEST_TOKEN_POOL, TEST_TOKEN_POOL_SUPPLY, TEST_TOKEN_YIELD, TEST_TOKEN_YIELD_SUPPLY,
};
use crate::testing::mock_adapter::Cw20HookMsg;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::mock_tax::MockTax;
use crate::testing::mock_token::{balances_to_map, MockToken};
use crate::testing::utils;
use cosmwasm_bignumber::{Decimal256, Uint256};
use pylon_utils::tax::deduct_tax;
use std::ops::{Div, Mul, Sub};
use std::str::FromStr;

#[test]
fn earn() {
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
    let beneficiary = mock_env(TEST_BENEFICIARY, &[]);

    let msg = HandleMsg::Earn {};
    let res = contract::handle(&mut deps, beneficiary.clone(), msg).unwrap();
    let reward = deduct_tax(
        &deps,
        Coin {
            denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
            amount: Uint128::from(TEST_TOKEN_POOL_SUPPLY),
        },
    )
    .unwrap();
    assert_eq!(res.data, None, "should be None");
    assert_eq!(
        res.messages,
        vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: HumanAddr::from(TEST_ADAPTER_TARGET),
                msg: to_binary(&Cw20HandleMsg::Send {
                    contract: HumanAddr::from(TEST_TOKEN_YIELD),
                    amount: Uint256::from(reward.amount)
                        .div(Decimal256::from_str(TEST_ADAPTER_EXCHANGE_RATE).unwrap())
                        .into(),
                    msg: Option::from(to_binary(&Cw20HookMsg::RedeemStable {}).unwrap()),
                })
                .unwrap(),
                send: vec![]
            }),
            CosmosMsg::Bank(BankMsg::Send {
                from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
                to_address: HumanAddr::from(TEST_BENEFICIARY),
                amount: vec![deduct_tax(
                    &deps,
                    Coin {
                        denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
                        amount: reward
                            .amount
                            .mul(Decimal::percent(100 - TEST_FACTORY_FEE_RATE))
                    }
                )
                .unwrap()]
            }),
            CosmosMsg::Bank(BankMsg::Send {
                from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
                to_address: HumanAddr::from(TEST_FACTORY_FEE_COLLECTOR),
                amount: vec![deduct_tax(
                    &deps,
                    Coin {
                        denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
                        amount: reward.amount.mul(Decimal::percent(TEST_FACTORY_FEE_RATE))
                    }
                )
                .unwrap()]
            }),
        ]
    );
}
