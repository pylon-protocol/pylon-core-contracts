use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    from_binary, to_binary, Api, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier,
    StdError, StdResult, Storage, WasmMsg,
};
use cw20::Cw20ReceiveMsg;
use pylon_gateway::pool_msg::{Cw20HookMsg, HandleMsg};

use crate::state::config;

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<HandleResponse> {
    let sender = env.message.sender.clone();

    if let Some(msg) = cw20_msg.msg {
        match from_binary(&msg)? {
            Cw20HookMsg::Deposit {} => {
                let config = config::read(&deps.storage)?;
                if sender.ne(&config.share_token) {
                    return Err(StdError::unauthorized());
                }

                deposit(deps, env, cw20_msg.sender, Uint256::from(cw20_msg.amount))
            }
        }
    } else {
        Err(StdError::generic_err("Staking: unsupported message"))
    }
}

pub fn deposit<S: Storage, A: Api, Q: Querier>(
    _: &Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.clone(),
                msg: to_binary(&HandleMsg::Update {
                    target: Option::Some(sender.clone()),
                })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address,
                msg: to_binary(&HandleMsg::DepositInternal { sender, amount })?,
                send: vec![],
            }),
        ],
        log: vec![],
        data: None,
    })
}

pub fn withdraw<S: Storage, A: Api, Q: Querier>(
    _: &Extern<S, A, Q>,
    env: Env,
    amount: Uint256,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.clone(),
                msg: to_binary(&HandleMsg::Update {
                    target: Option::Some(env.message.sender.clone()),
                })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address,
                msg: to_binary(&HandleMsg::WithdrawInternal {
                    sender: env.message.sender,
                    amount,
                })?,
                send: vec![],
            }),
        ],
        log: vec![],
        data: None,
    })
}

pub fn claim<S: Storage, A: Api, Q: Querier>(
    _: &Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address.clone(),
                msg: to_binary(&HandleMsg::Update {
                    target: Option::Some(env.message.sender.clone()),
                })?,
                send: vec![],
            }),
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: env.contract.address,
                msg: to_binary(&HandleMsg::ClaimInternal {
                    sender: env.message.sender,
                })?,
                send: vec![],
            }),
        ],
        log: vec![],
        data: None,
    })
}
