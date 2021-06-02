use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, CosmosMsg, Env, Extern, HandleResponse, InitResponse,
    MigrateResponse, MigrateResult, Querier, StdResult, Storage, WasmMsg,
};

use cw20::MinterResponse;
use terraswap::hook::InitHook as Cw20InitHook;
use terraswap::token::InitMsg as Cw20InitMsg;

use crate::handler::core as CoreHandler;
use crate::handler::query as QueryHandler;
use crate::{config, querier};
use pylon_core::pool_msg::{HandleMsg, InitMsg, MigrateMsg, QueryMsg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let sender = env.message.sender;
    let raw_sender = deps.api.canonical_address(&sender)?;

    let mut config = config::Config {
        this: deps.api.canonical_address(&env.contract.address)?,
        owner: raw_sender,
        beneficiary: deps.api.canonical_address(&msg.beneficiary)?,
        fee_collector: deps.api.canonical_address(&msg.fee_collector)?,
        exchange_rate_feeder: deps.api.canonical_address(&msg.exchange_rate_feeder)?,
        moneymarket: deps.api.canonical_address(&msg.moneymarket)?,
        stable_denom: String::default(),
        atoken: CanonicalAddr::default(),
        dp_token: CanonicalAddr::default(),
    };

    let market_config = querier::anchor::config(deps, &config.moneymarket)?;

    config.stable_denom = market_config.stable_denom.clone();
    config.atoken = deps.api.canonical_address(&market_config.aterra_contract)?;

    config::store(&mut deps.storage, &config)?;

    Ok(InitResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: msg.dp_code_id,
            send: vec![],
            label: None,
            msg: to_binary(&Cw20InitMsg {
                name: format!("Pylon Deposit Token {}", msg.pool_name),
                symbol: "DPv1".to_string(),
                decimals: 6u8,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.clone(),
                    cap: None,
                }),
                init_hook: Some(Cw20InitHook {
                    contract_addr: env.contract.address.clone(),
                    msg: to_binary(&HandleMsg::RegisterDPToken {})?,
                }),
            })?,
        })],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Receive(msg) => CoreHandler::receive(deps, env, msg),
        HandleMsg::Deposit {} => CoreHandler::deposit(deps, env),
        HandleMsg::ClaimReward {} => CoreHandler::claim_reward(deps, env),
        HandleMsg::RegisterDPToken {} => CoreHandler::register_dp_token(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::DepositAmountOf { owner } => QueryHandler::deposit_amount(deps, owner), // dp_token.balanceOf(msg.sender)
        QueryMsg::TotalDepositAmount {} => QueryHandler::total_deposit_amount(deps), // dp_token.totalSupply()
        QueryMsg::GetBeneficiary {} => QueryHandler::beneficiary(deps), // config.beneficiary
        QueryMsg::GetMoneyMarket {} => QueryHandler::money_market(deps),
        QueryMsg::GetStableDenom {} => QueryHandler::stable_denom(deps),
        QueryMsg::GetAToken {} => QueryHandler::anchor_token(deps),
        QueryMsg::GetDPToken {} => QueryHandler::dp_token(deps),
        QueryMsg::GetClaimableReward {} => QueryHandler::claimable_reward(deps), // config.strategy.reward()
    }
}

pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> MigrateResult {
    Ok(MigrateResponse::default())
}
