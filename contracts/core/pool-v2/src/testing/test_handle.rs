use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, BankMsg, Coin, CosmosMsg, Decimal, HumanAddr, Uint128, WasmMsg};
use cw20::{Cw20HandleMsg, Cw20ReceiveMsg};
use pylon_core::pool_v2_msg::{Cw20HookMsg, HandleMsg};
use pylon_token::collector::HandleMsg as CollectorHandleMsg;
use pylon_utils::tax::deduct_tax;
use std::ops::{Div, Mul};
use std::str::FromStr;

use crate::contract;
use crate::testing::constants::{
    TEST_ADAPTER_EXCHANGE_RATE, TEST_ADAPTER_INPUT_DENOM, TEST_ADAPTER_TARGET, TEST_BENEFICIARY,
    TEST_FACTORY_FEE_COLLECTOR, TEST_FACTORY_FEE_RATE, TEST_TOKEN_POOL, TEST_TOKEN_POOL_SUPPLY,
    TEST_TOKEN_YIELD, TEST_TOKEN_YIELD_SUPPLY, TEST_USER,
};
use crate::testing::mock_adapter::{Cw20HookMsg as AdapterHookMsg, HandleMsg as AdapterHandlerMsg};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::mock_tax::MockTax;
use crate::testing::mock_token::{balances_to_map, MockToken};
use crate::testing::utils;

#[test]
fn handle_deposit() {
    let mut deps = mock_dependencies(20, &[]);
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
        amount: Uint128::from(10000000 as u64),
    };
    let coin_with_tax = deduct_tax(&deps, coin.clone()).unwrap();
    let user = mock_env(TEST_USER, &[coin.clone()]);

    let msg = HandleMsg::Deposit {};
    let res = contract::handle(&mut deps, user.clone(), msg).unwrap();
    assert_eq!(res.data, None, "should be none");
    assert_eq!(
        res.messages,
        vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: HumanAddr::from(TEST_ADAPTER_TARGET),
                msg: to_binary(&AdapterHandlerMsg::DepositStable {}).unwrap(),
                send: vec![coin.clone()],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: HumanAddr::from(TEST_TOKEN_POOL),
                msg: to_binary(&Cw20HandleMsg::Mint {
                    recipient: user.message.sender.clone(),
                    amount: coin_with_tax.amount.clone(),
                })
                .unwrap(),
                send: vec![]
            }),
        ]
    );
}

#[test]
fn handle_redeem() {
    let mut deps = mock_dependencies(20, &[]);
    deps.querier.with_tax(MockTax::new(
        Decimal::percent(1),
        &[(
            &TEST_ADAPTER_INPUT_DENOM.to_string(),
            &Uint128::from(1000000u128),
        )],
    ));
    let _ = utils::initialize(&mut deps);

    let amount = Uint128::from(10000000 as u64);
    let amount_with_tax = deduct_tax(
        &deps,
        Coin {
            denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
            amount: deduct_tax(
                &deps,
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
    let user = mock_env(TEST_TOKEN_POOL, &[]);

    let msg = HandleMsg::Receive(Cw20ReceiveMsg {
        sender: HumanAddr::from(TEST_USER),
        amount: amount.clone(),
        msg: Some(to_binary(&Cw20HookMsg::Redeem {}).unwrap()),
    });
    let res = contract::handle(&mut deps, user.clone(), msg).unwrap();
    assert_eq!(res.data, None, "should be none");
    assert_eq!(
        res.messages,
        vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: HumanAddr::from(TEST_TOKEN_POOL),
                msg: to_binary(&Cw20HandleMsg::Burn {
                    amount: amount.clone()
                })
                .unwrap(),
                send: vec![]
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: HumanAddr::from(TEST_ADAPTER_TARGET),
                msg: to_binary(&Cw20HandleMsg::Send {
                    contract: HumanAddr::from(TEST_TOKEN_YIELD),
                    amount: Uint256::from(amount)
                        .div(Decimal256::from_str(TEST_ADAPTER_EXCHANGE_RATE).unwrap())
                        .into(),
                    msg: Some(to_binary(&AdapterHookMsg::RedeemStable {}).unwrap())
                })
                .unwrap(),
                send: vec![]
            }),
            CosmosMsg::Bank(BankMsg::Send {
                from_address: HumanAddr::from(MOCK_CONTRACT_ADDR),
                to_address: HumanAddr::from(TEST_USER),
                amount: vec![Coin {
                    denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
                    amount: amount_with_tax.clone(),
                }],
            })
        ]
    )
}

#[test]
fn handle_earn() {
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
                    msg: Option::from(to_binary(&AdapterHookMsg::RedeemStable {}).unwrap()),
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
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: HumanAddr::from(TEST_FACTORY_FEE_COLLECTOR),
                msg: to_binary(&CollectorHandleMsg::Sweep {
                    denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
                })
                .unwrap(),
                send: vec![]
            })
        ]
    );
}
