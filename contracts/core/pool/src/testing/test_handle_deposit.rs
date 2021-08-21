use cosmwasm_std::testing::mock_env;
use cosmwasm_std::{to_binary, Coin, CosmosMsg, Decimal, HumanAddr, Uint128, WasmMsg};
use cw20::Cw20HandleMsg;
use pylon_core::pool_msg::HandleMsg;
use pylon_utils::tax::deduct_tax;

use crate::contract;
use crate::testing::constants::{
    TEST_ADAPTER_INPUT_DENOM, TEST_ADAPTER_TARGET, TEST_TOKEN_POOL, TEST_USER,
};
use crate::testing::mock_adapter::HandleMsg as AdapterHandlerMsg;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::mock_tax::MockTax;
use crate::testing::utils;

#[test]
fn deposit() {
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
