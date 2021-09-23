use cosmwasm_std::testing::{mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{to_binary, Api, CanonicalAddr, CosmosMsg, HumanAddr, WasmMsg};
use cw20::MinterResponse;
use pylon_core::factory_msg::HandleMsg as FactoryHandleMsg;
use pylon_core::pool_v2_msg::HandleMsg as PoolHandleMsg;
use terraswap::hook::InitHook as Cw20InitHook;
use terraswap::token::InitMsg as Cw20InitMsg;

use crate::contract;
use crate::state::config;
use crate::testing::constants::{
    TEST_ADAPTER, TEST_ADAPTER_INPUT_DENOM, TEST_BENEFICIARY, TEST_FACTORY, TEST_POOL_ID,
    TEST_POOL_NAME, TEST_TOKEN_CODE_ID, TEST_TOKEN_YIELD,
};
use crate::testing::mock_querier::mock_dependencies;
use crate::testing::utils;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = utils::init_msg();
    let env = mock_env(TEST_FACTORY, &[]);
    let res = contract::init(&mut deps, env.clone(), msg).unwrap();
    assert_eq!(
        res.messages,
        vec![
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: TEST_TOKEN_CODE_ID,
                msg: to_binary(&Cw20InitMsg {
                    name: "Pylon Deposit Pool Token".to_string(),
                    symbol: "DPv1".to_string(),
                    decimals: 6u8,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.clone(),
                        cap: None
                    }),
                    init_hook: Some(Cw20InitHook {
                        contract_addr: env.contract.address,
                        msg: to_binary(&PoolHandleMsg::RegisterDPToken {}).unwrap(),
                    })
                })
                .unwrap(),
                send: vec![],
                label: None
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: HumanAddr::from(TEST_FACTORY),
                send: vec![],
                msg: to_binary(&FactoryHandleMsg::RegisterPool {
                    pool_id: TEST_POOL_ID
                })
                .unwrap()
            })
        ]
    );

    let config = config::read(&deps.storage).unwrap();
    assert_eq!(
        config,
        config::Config {
            id: TEST_POOL_ID,
            name: TEST_POOL_NAME.to_string(),
            this: deps
                .api
                .canonical_address(&HumanAddr::from(MOCK_CONTRACT_ADDR))
                .unwrap(),
            factory: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_FACTORY))
                .unwrap(),
            beneficiary: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_BENEFICIARY))
                .unwrap(),
            yield_adapter: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_ADAPTER))
                .unwrap(),
            input_denom: TEST_ADAPTER_INPUT_DENOM.to_string(),
            yield_token: deps
                .api
                .canonical_address(&HumanAddr::from(TEST_TOKEN_YIELD))
                .unwrap(),
            dp_token: CanonicalAddr::default(),
        }
    )
}
