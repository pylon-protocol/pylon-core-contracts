use crate::testing::constants::{
    TEST_POOL_TOKEN, TEST_POOL_TOKEN_SUPPLY, TEST_YIELD_TOKEN, TEST_YIELD_TOKEN_SUPPLY,
};
use cosmwasm_std::{to_binary, HumanAddr, QuerierResult, SystemError, Uint128};
use cw20::{BalanceResponse, Cw20QueryMsg, TokenInfoResponse};
use std::collections::HashMap;

#[derive(Clone)]
pub struct MockToken {
    pub infos: HashMap<HumanAddr, TokenInfoResponse>,
    pub balances: HashMap<HumanAddr, HashMap<HumanAddr, Uint128>>,
}

impl Default for MockToken {
    fn default() -> Self {
        MockToken::new(
            &[
                (
                    &TEST_YIELD_TOKEN.to_string(),
                    TokenInfoResponse {
                        name: TEST_YIELD_TOKEN.to_string(),
                        symbol: "TNT".to_string(),
                        decimals: 6u8,
                        total_supply: Uint128::from(TEST_YIELD_TOKEN_SUPPLY),
                    },
                ),
                (
                    &TEST_POOL_TOKEN.to_string(),
                    TokenInfoResponse {
                        name: TEST_POOL_TOKEN.to_string(),
                        symbol: "TNT".to_string(),
                        decimals: 6u8,
                        total_supply: Uint128::from(TEST_POOL_TOKEN_SUPPLY),
                    },
                ),
            ],
            &[],
        )
    }
}

impl MockToken {
    pub fn handle_query(&self, token: &HumanAddr, msg: Cw20QueryMsg) -> QuerierResult {
        match msg {
            Cw20QueryMsg::Balance { address } => Ok(to_binary(&BalanceResponse {
                balance: match self.balances.get(token) {
                    Some(token_balance_map) => match token_balance_map.get(&address) {
                        Some(amount) => *amount,
                        None => Uint128::zero(),
                    },
                    None => Uint128::zero(),
                },
            })),
            Cw20QueryMsg::TokenInfo {} => match self.infos.get(token) {
                Some(info) => Ok(to_binary(info)),
                None => Err(SystemError::UnsupportedRequest {
                    kind: token.to_string(),
                }),
            },
            _ => Err(SystemError::UnsupportedRequest {
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
}

pub fn infos_to_map(
    infos: &[(&String, TokenInfoResponse)],
) -> HashMap<HumanAddr, TokenInfoResponse> {
    let mut info_map: HashMap<HumanAddr, TokenInfoResponse> = HashMap::new();
    for (token, token_info) in infos.iter() {
        info_map.insert(HumanAddr::from(token.to_string()), token_info.clone());
    }

    info_map
}

pub fn balances_to_map(
    balances: &[(&String, &[(&String, &Uint128)])],
) -> HashMap<HumanAddr, HashMap<HumanAddr, Uint128>> {
    let mut token_map: HashMap<HumanAddr, HashMap<HumanAddr, Uint128>> = HashMap::new();
    for (token, token_balances) in balances.iter() {
        let mut balance_map: HashMap<HumanAddr, Uint128> = HashMap::new();
        for (owner, balance) in token_balances.iter() {
            balance_map.insert(HumanAddr::from(owner.to_string()), **balance);
        }
        token_map.insert(HumanAddr::from(token.to_string()), balance_map);
    }

    token_map
}
