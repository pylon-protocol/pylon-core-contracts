use cosmwasm_std::{to_binary, ContractResult, QuerierResult, SystemError, SystemResult, Uint128};
use cw20::{BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use std::collections::HashMap;

#[derive(Clone)]
pub struct MockToken {
    pub infos: HashMap<String, TokenInfoResponse>,
    pub balances: HashMap<String, HashMap<String, Uint128>>,
}

impl Default for MockToken {
    fn default() -> Self {
        MockToken::new(&[], &[])
    }
}

impl MockToken {
    pub fn handle_query(&self, token: &str, msg: Cw20QueryMsg) -> QuerierResult {
        match msg {
            Cw20QueryMsg::Balance { address } => SystemResult::Ok(ContractResult::Ok(
                to_binary(&BalanceResponse {
                    balance: match self.balances.get(token) {
                        Some(token_balance_map) => match token_balance_map.get(&address) {
                            Some(amount) => *amount,
                            None => Uint128::zero(),
                        },
                        None => Uint128::zero(),
                    },
                })
                .unwrap(),
            )),
            Cw20QueryMsg::TokenInfo {} => match self.infos.get(token) {
                Some(info) => SystemResult::Ok(ContractResult::Ok(to_binary(info).unwrap())),
                None => SystemResult::Err(SystemError::UnsupportedRequest {
                    kind: token.to_string(),
                }),
            },
            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: stringify!(msg).to_string(),
            }),
        }
    }
}

impl MockToken {
    pub fn new(
        infos: &[(&String, TokenInfoResponse)],
        balances: &[(&String, &[(&String, &Uint128)])],
    ) -> Self {
        MockToken {
            infos: infos_to_map(infos),
            balances: balances_to_map(balances),
        }
    }

    #[allow(dead_code)]
    pub fn with_infos(&mut self, infos: &[(&String, TokenInfoResponse)]) {
        self.infos = infos_to_map(infos);
    }

    pub fn with_balances(&mut self, balances: &[(&String, &[(&String, &Uint128)])]) {
        self.balances = balances_to_map(balances);
    }
}

pub fn infos_to_map(infos: &[(&String, TokenInfoResponse)]) -> HashMap<String, TokenInfoResponse> {
    let mut info_map: HashMap<String, TokenInfoResponse> = HashMap::new();
    for (token, token_info) in infos.iter() {
        info_map.insert(token.to_string(), token_info.clone());
    }

    info_map
}

pub fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<String, HashMap<String, Uint128>> {
    let mut token_map: HashMap<String, HashMap<String, Uint128>> = HashMap::new();
    for (token, token_balances) in balances.iter() {
        let mut balance_map: HashMap<String, Uint128> = HashMap::new();
        for (owner, balance) in token_balances.iter() {
            balance_map.insert(owner.to_string(), **balance);
        }
        token_map.insert(token.to_string(), balance_map);
    }

    token_map
}
