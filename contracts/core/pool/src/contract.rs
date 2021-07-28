use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, CosmosMsg, Env, Extern, HandleResponse, InitResponse,
    MigrateResponse, MigrateResult, Querier, QueryRequest, StdResult, Storage, WasmMsg, WasmQuery,
};

use cw20::MinterResponse;
use terraswap::hook::InitHook as Cw20InitHook;
use terraswap::token::InitMsg as Cw20InitMsg;

use crate::handler::core as CoreHandler;
use crate::handler::query as QueryHandler;
use crate::state::config;

use pylon_core::adapter::{ConfigResponse, QueryMsg as AdapterQueryMsg};
use pylon_core::factory_msg::HandleMsg as FactoryHandleMsg;
use pylon_core::pool_msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let adapter_config: ConfigResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: msg.yield_adapter.clone(),
            msg: to_binary(&AdapterQueryMsg::Config {})?,
        }))?;

    let config = config::Config {
        id: msg.pool_id.clone(),
        name: msg.pool_name.clone(),
        this: deps.api.canonical_address(&env.contract.address)?,
        factory: deps.api.canonical_address(&env.message.sender)?,
        beneficiary: deps.api.canonical_address(&msg.beneficiary)?,
        fee_collector: deps.api.canonical_address(&msg.fee_collector)?,
        yield_adapter: deps.api.canonical_address(&msg.yield_adapter)?,
        input_denom: adapter_config.input_denom,
        yield_token: deps.api.canonical_address(&adapter_config.yield_token)?,
        dp_token: CanonicalAddr::default(),
    };

    config::store(&mut deps.storage, &config)?;

    Ok(InitResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: msg.dp_code_id,
                send: vec![],
                label: None,
                msg: to_binary(&Cw20InitMsg {
                    name: "Pylon Deposit Token".to_string(),
                    symbol: "DPv1".to_string(),
                    decimals: 6u8,
                    initial_balances: vec![],
                    mint: Some(MinterResponse {
                        minter: env.contract.address.clone(),
                        cap: None,
                    }),
                    init_hook: Some(Cw20InitHook {
                        contract_addr: env.contract.address,
                        msg: to_binary(&HandleMsg::RegisterDPToken {})?,
                    }),
                })?,
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.human_address(&config.factory)?,
                send: vec![],
                msg: to_binary(&FactoryHandleMsg::RegisterPool {
                    pool_id: msg.pool_id,
                })?,
            }),
        ],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::RegisterDPToken {} => CoreHandler::register_dp_token(deps, env),
        HandleMsg::Receive(msg) => CoreHandler::receive(deps, env, msg),
        HandleMsg::Deposit {} => CoreHandler::deposit(deps, env),
        HandleMsg::Earn {} => CoreHandler::earn(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => QueryHandler::config(deps),
        QueryMsg::DepositAmountOf { owner } => QueryHandler::deposit_amount(deps, owner),
        QueryMsg::TotalDepositAmount {} => QueryHandler::total_deposit_amount(deps),
        QueryMsg::ClaimableReward {} => QueryHandler::claimable_reward(deps),
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
