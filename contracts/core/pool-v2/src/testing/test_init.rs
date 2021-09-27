use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{to_binary, CosmosMsg, SubMsg, WasmMsg};
use cw20::MinterResponse;
use pylon_core::factory_msg::ExecuteMsg as FactoryExecuteMsg;
use pylon_core::test_constant::*;
use terraswap::token::InstantiateMsg as Cw20InstantiateMsg;

use crate::contract;
use crate::state::config;
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = utils::init_msg();
    let env = mock_env();
    let info = mock_info(TEST_FACTORY, &[]);
    let res = contract::instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            SubMsg::reply_on_success(
                CosmosMsg::Wasm(WasmMsg::Instantiate {
                    admin: None,
                    code_id: TEST_TOKEN_CODE_ID,
                    msg: to_binary(&Cw20InstantiateMsg {
                        name: "Pylon Deposit Pool Token".to_string(),
                        symbol: "DPvTwo".to_string(),
                        decimals: 6u8,
                        initial_balances: vec![],
                        mint: Some(MinterResponse {
                            minter: env.contract.address.to_string(),
                            cap: None
                        }),
                    })
                    .unwrap(),
                    label: "".to_string(),
                    funds: vec![]
                }),
                1
            ),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: TEST_FACTORY.to_string(),
                msg: to_binary(&FactoryExecuteMsg::RegisterPool {
                    pool_id: TEST_POOL_ID
                })
                .unwrap(),
                funds: vec![],
            }))
        ]
    );

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config,
        config::Config {
            id: TEST_POOL_ID,
            name: TEST_POOL_NAME.to_string(),
            factory: TEST_FACTORY.to_string(),
            beneficiary: TEST_BENEFICIARY.to_string(),
            yield_adapter: TEST_ADAPTER.to_string(),
            input_denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
            yield_token: TEST_TOKEN_YIELD.to_string(),
            dp_token: "".to_string(),
        }
    )
}
