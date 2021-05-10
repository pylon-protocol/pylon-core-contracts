use cosmwasm_std::{
    to_binary, Api, Binary, CanonicalAddr, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::{Cw20CoinHuman, MinterResponse};
use terraswap::hook::InitHook as TokenInitHook;
use terraswap::token::InitMsg as TokenInitMsg;

use crate::config::{read_config, store_config, Config};
use crate::handler_exec::{
    handle_claim_reward, handle_deposit, handle_receive, handle_redeem, handle_register_dp_token,
};
use crate::handler_query::{
    query_anchor_token, query_beneficiary, query_claimable_reward, query_deposit_amount,
    query_dp_token, query_money_market, query_stable_denom, query_total_deposit_amount,
};
use crate::lib_anchor::market_config;
use crate::msg::{HandleMsg, InitMsg, QueryMsg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let sender = env.message.sender;
    let raw_sender = deps.api.canonical_address(&sender)?;

    let mut config = Config {
        this: deps.api.canonical_address(&env.contract.address)?,
        owner: raw_sender,
        beneficiary: deps.api.canonical_address(&msg.beneficiary)?,
        moneymarket: deps.api.canonical_address(&msg.moneymarket)?,
        stable_denom: String::default(),
        atoken: CanonicalAddr::default(),
        dp_token: CanonicalAddr::default(),
    };

    let market_config = market_config(deps, &config.moneymarket)?;

    config.stable_denom = market_config.stable_denom.clone();
    config.atoken = deps.api.canonical_address(&market_config.aterra_contract)?;

    store_config(&mut deps.storage, &config);

    Ok(InitResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: msg.dp_code_id,
            send: vec![],
            label: None,
            msg: to_binary(&TokenInitMsg {
                name: format!("Pylon Deposit Token {}", msg.pool_name),
                symbol: "DPv1".to_string(),
                decimals: 6u8,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.clone(),
                    cap: None,
                }),
                init_hook: Some(TokenInitHook {
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
        HandleMsg::Receive(msg) => handle_receive(deps, env, msg),
        HandleMsg::Deposit {} => handle_deposit(deps, env),
        HandleMsg::ClaimReward {} => handle_claim_reward(deps, env),
        HandleMsg::RegisterDPToken {} => handle_register_dp_token(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::DepositAmountOf { owner } => to_binary(&query_deposit_amount(deps, owner)?), // dp_token.balanceOf(msg.sender)
        QueryMsg::TotalDepositAmount {} => to_binary(&query_total_deposit_amount(deps)?), // dp_token.totalSupply()
        QueryMsg::GetBeneficiary {} => to_binary(&query_beneficiary(deps)?), // config.beneficiary
        QueryMsg::GetMoneyMarket {} => to_binary(&query_money_market(deps)?),
        QueryMsg::GetStableDenom {} => to_binary(&query_stable_denom(deps)?),
        QueryMsg::GetAToken {} => to_binary(&query_anchor_token(deps)?),
        QueryMsg::GetDPToken {} => to_binary(&query_dp_token(deps)?),
        QueryMsg::GetClaimableReward {} => to_binary(&query_claimable_reward(deps)?), // config.strategy.reward()
    }
}
