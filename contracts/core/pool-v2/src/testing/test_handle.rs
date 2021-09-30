use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, BankMsg, Coin, CosmosMsg, Decimal, SubMsg, Uint128, WasmMsg};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use pylon_core::mock_adapter::{Cw20HookMsg as AdapterHookMsg, ExecuteMsg as AdapterExecuteMsg};
use pylon_core::pool_v2_msg::{Cw20HookMsg, ExecuteMsg};
use pylon_core::test_constant::*;
use pylon_token::collector::ExecuteMsg as CollectorExecuteMsg;
use pylon_utils::mock_tax::MockTax;
use pylon_utils::tax::deduct_tax;
use std::ops::{Div, Mul};
use std::str::FromStr;

use crate::contract;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn handle_deposit() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.with_tax(MockTax::new(
        Decimal::percent(1),
        &[(
            &TEST_ADAPTER_INPUT_DENOM.to_string(),
            &Uint128::from(1000000u128),
        )],
    ));
    let _ = utils::initialize(&mut deps);

    let coin = Coin {
        denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
        amount: Uint128::from(10000000_u64),
    };
    let coin_with_tax = deduct_tax(deps.as_ref(), coin.clone()).unwrap();
    let user = mock_info(TEST_USER, &[coin.clone()]);

    let msg = ExecuteMsg::Deposit {};
    let res = contract::execute(deps.as_mut(), mock_env(), user.clone(), msg).unwrap();
    assert_eq!(res.data, None, "should be none");
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_ADAPTER_TARGET.to_string(),
                msg: to_binary(&AdapterExecuteMsg::DepositStable {}).unwrap(),
                funds: vec![coin],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_TOKEN_POOL.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Mint {
                    recipient: user.sender.to_string(),
                    amount: coin_with_tax.amount,
                })
                .unwrap(),
                funds: vec![]
            })),
        ]
    );
}

#[test]
fn handle_redeem() {
    let mut deps = mock_dependencies(&[]);
    deps.querier.with_tax(MockTax::new(
        Decimal::percent(1),
        &[(
            &TEST_ADAPTER_INPUT_DENOM.to_string(),
            &Uint128::from(1000000u128),
        )],
    ));
    let _ = utils::initialize(&mut deps);

    let amount = Uint128::from(10000000_u64);
    let amount_with_tax = deduct_tax(
        deps.as_ref(),
        Coin {
            denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
            amount: deduct_tax(
                deps.as_ref(),
                Coin {
                    denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
                    amount,
                },
            )
            .unwrap()
            .amount,
        },
    )
    .unwrap()
    .amount;
    let user = mock_info(TEST_TOKEN_POOL, &[]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_USER.to_string(),
        amount,
        msg: to_binary(&Cw20HookMsg::Redeem {}).unwrap(),
    });
    let res = contract::execute(deps.as_mut(), mock_env(), user, msg).unwrap();
    assert_eq!(res.data, None, "should be none");
    assert_eq!(
        res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_TOKEN_POOL.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Burn { amount }).unwrap(),
                funds: vec![]
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_ADAPTER_TARGET.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: TEST_TOKEN_YIELD.to_string(),
                    amount: Uint256::from(amount)
                        .div(Decimal256::from_str(TEST_ADAPTER_EXCHANGE_RATE).unwrap())
                        .into(),
                    msg: to_binary(&AdapterHookMsg::RedeemStable {}).unwrap()
                })
                .unwrap(),
                funds: vec![]
            })),
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: TEST_USER.to_string(),
                amount: vec![Coin {
                    denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
                    amount: amount_with_tax,
                }],
            }))
        ]
    )
}

#[test]
fn handle_earn() {
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
    let beneficiary = mock_info(TEST_BENEFICIARY, &[]);

    let msg = ExecuteMsg::Earn {};
    let res = contract::execute(deps.as_mut(), mock_env(), beneficiary, msg).unwrap();
    let reward = deduct_tax(
        deps.as_ref(),
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
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_ADAPTER_TARGET.to_string(),
                msg: to_binary(&Cw20ExecuteMsg::Send {
                    contract: TEST_TOKEN_YIELD.to_string(),
                    amount: Uint256::from(reward.amount)
                        .div(Decimal256::from_str(TEST_ADAPTER_EXCHANGE_RATE).unwrap())
                        .into(),
                    msg: to_binary(&AdapterHookMsg::RedeemStable {}).unwrap(),
                })
                .unwrap(),
                funds: vec![]
            })),
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: TEST_BENEFICIARY.to_string(),
                amount: vec![deduct_tax(
                    deps.as_ref(),
                    Coin {
                        denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
                        amount: reward
                            .amount
                            .mul(Decimal::percent(100 - TEST_FACTORY_FEE_RATE))
                    }
                )
                .unwrap()]
            })),
            SubMsg::new(CosmosMsg::Bank(BankMsg::Send {
                to_address: TEST_FACTORY_FEE_COLLECTOR.to_string(),
                amount: vec![deduct_tax(
                    deps.as_ref(),
                    Coin {
                        denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
                        amount: reward.amount.mul(Decimal::percent(TEST_FACTORY_FEE_RATE))
                    }
                )
                .unwrap()]
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_FACTORY_FEE_COLLECTOR.to_string(),
                msg: to_binary(&CollectorExecuteMsg::Sweep {
                    denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
                })
                .unwrap(),
                funds: vec![]
            }))
        ]
    );
}
