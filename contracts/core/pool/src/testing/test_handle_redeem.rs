use cosmwasm_bignumber::{Decimal256, Uint256};
use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, BankMsg, Coin, CosmosMsg, Decimal, HumanAddr, Uint128, WasmMsg};
use cw20::{Cw20HandleMsg, Cw20ReceiveMsg};
use pylon_core::pool_msg::{Cw20HookMsg, HandleMsg};
use pylon_utils::tax::deduct_tax;
use std::ops::Div;
use std::str::FromStr;

use crate::contract;
use crate::testing::constants::{
    TEST_ADAPTER_EXCHANGE_RATE, TEST_ADAPTER_INPUT_DENOM, TEST_ADAPTER_TARGET, TEST_TOKEN_POOL,
    TEST_TOKEN_YIELD, TEST_USER,
};
use crate::testing::mock_adapter::Cw20HookMsg as AdapterHookMsg;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::mock_tax::MockTax;
use crate::testing::utils;

#[test]
fn redeem() {
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
            amount,
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
