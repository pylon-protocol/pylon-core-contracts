use cosmwasm_bignumber::Uint256;
use cosmwasm_std::{
    coin, from_binary, to_binary, Api, BankMsg, Binary, CosmosMsg, Env, Extern, HandleResponse,
    HumanAddr, InitResponse, MigrateResponse, Querier, StdResult, Storage, WasmMsg,
};
use cw20::{Cw20HandleMsg, MinterResponse};
use pylon_testing::market_msg::{Cw20HookMsg, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use pylon_testing::market_resp::ConfigResponse;
use std::ops::{Div, Mul};
use terraswap::hook::InitHook as Cw20InitHook;
use terraswap::token::InitMsg as Cw20InitMsg;

use crate::config;
use crate::error::ContractError;

#[allow(dead_code)]
pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InstantiateMsg,
) -> Result<InitResponse, ContractError> {
    config::store(
        &mut deps.storage,
        &config::Config {
            owner: env.message.sender.to_string(),
            input_denom: msg.input_denom,
            output_token: "".to_string(),
            exchange_rate: msg.exchange_rate,
        },
    )
    .unwrap();
    Ok(InitResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: msg.token_code_id,
            msg: to_binary(&Cw20InitMsg {
                name: "MarketOutputToken".to_string(),
                symbol: "MOT".to_string(), // naming sucks
                decimals: 6u8,
                initial_balances: vec![],
                mint: Some(MinterResponse {
                    minter: env.contract.address.clone(),
                    cap: None,
                }),
                init_hook: Option::from(Cw20InitHook {
                    msg: to_binary(&ExecuteMsg::RegisterOutputToken {}).unwrap(),
                    contract_addr: env.contract.address,
                }),
            })?,
            send: vec![],
            label: None,
        })],
        log: vec![],
    })
}

#[allow(dead_code)]
pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: ExecuteMsg,
) -> Result<HandleResponse, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => {
            let config = config::read(&deps.storage).unwrap();
            if config.output_token != env.message.sender.to_string() {
                return Err(ContractError::Unauthorized {
                    action: "cw20_receive".to_string(),
                    expected: config.output_token,
                    actual: env.message.sender.to_string(),
                });
            }
            if let Some(bin_msg) = cw20_msg.msg {
                match from_binary(&bin_msg) {
                    Ok(Cw20HookMsg::RedeemStable {}) => {
                        let return_amount =
                            Uint256::from(cw20_msg.amount).mul(config.exchange_rate);
                        Ok(HandleResponse {
                            messages: vec![
                                CosmosMsg::Wasm(WasmMsg::Execute {
                                    contract_addr: HumanAddr::from(config.output_token),
                                    msg: to_binary(&Cw20HandleMsg::Burn {
                                        amount: cw20_msg.amount,
                                    })
                                    .unwrap(),
                                    send: vec![],
                                }),
                                CosmosMsg::Bank(BankMsg::Send {
                                    from_address: env.contract.address,
                                    to_address: cw20_msg.sender,
                                    amount: vec![coin(
                                        return_amount.into(),
                                        config.input_denom.as_str(),
                                    )],
                                }),
                            ],
                            log: vec![],
                            data: None,
                        })
                    }
                    _ => Err(ContractError::UnsupportedReceiveMsg {
                        typ: stringify!(unmarshalled).to_string(),
                    }),
                }
            } else {
                Err(ContractError::UnsupportedReceiveMsg {
                    typ: stringify!(cw20_msg).to_string(),
                })
            }
        }
        ExecuteMsg::DepositStable {} => {
            let config = config::read(&deps.storage).unwrap();
            let received = env
                .message
                .sent_funds
                .iter()
                .find(|c| c.denom == config.input_denom)
                .map(|c| Uint256::from(c.amount))
                .unwrap_or_else(Uint256::zero);
            if received.is_zero() {
                return Err(ContractError::NotAllowZeroAmount {});
            }
            let return_amount = received.div(config.exchange_rate);

            Ok(HandleResponse {
                messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: HumanAddr::from(config.output_token),
                    msg: to_binary(&Cw20HandleMsg::Mint {
                        recipient: env.message.sender,
                        amount: return_amount.into(),
                    })
                    .unwrap(),
                    send: vec![],
                })],
                log: vec![],
                data: None,
            })
        }
        ExecuteMsg::RegisterOutputToken {} => {
            let mut config = config::read(&deps.storage).unwrap();

            if !config.output_token.is_empty() {
                return Err(ContractError::Unauthorized {
                    action: "reply_init_output_token".to_string(),
                    expected: "<empty>".to_string(),
                    actual: config.output_token,
                });
            }
            config.output_token = env.message.sender.to_string();

            config::store(&mut deps.storage, &config).unwrap();
            Ok(HandleResponse::default())
        }
        ExecuteMsg::Configure { exchange_rate } => {
            let mut config = config::read(&deps.storage).unwrap();

            if config.owner != env.message.sender.to_string() {
                return Err(ContractError::Unauthorized {
                    action: "configure".to_string(),
                    expected: config.owner,
                    actual: env.message.sender.to_string(),
                });
            }
            config.exchange_rate = exchange_rate;

            config::store(&mut deps.storage, &config).unwrap();
            Ok(HandleResponse::default())
        }
    }
}

#[allow(dead_code)]
pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config = config::read(&deps.storage).unwrap();
            to_binary(&ConfigResponse {
                owner: config.owner,
                input_denom: config.input_denom,
                output_token: config.output_token,
                exchange_rate: config.exchange_rate,
            })
        }
    }
}

#[allow(dead_code)]
pub fn migrate<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<MigrateResponse, ContractError> {
    Ok(MigrateResponse::default())
}
