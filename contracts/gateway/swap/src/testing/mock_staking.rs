use cosmwasm_std::{to_binary, ContractResult, QuerierResult, SystemError, SystemResult, Uint128};
use pylon_token::gov::{QueryMsg, StakerResponse};
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct MockStaking {
    pub infos: HashMap<String, StakerResponse>,
}

impl MockStaking {
    pub fn handle_query(&self, msg: QueryMsg) -> QuerierResult {
        match msg {
            QueryMsg::Staker { address } => {
                let def = &StakerResponse {
                    balance: Uint128::zero(),
                    share: Uint128::zero(),
                    locked_balance: vec![],
                };
                SystemResult::Ok(ContractResult::Ok(to_binary(
                    match self.infos.get(&address) {
                        Some(info) => info,
                        None => def,
                    },
                )?))
            }
            _ => SystemResult::Err(SystemError::UnsupportedRequest {
                kind: stringify!(msg).to_string(),
            }),
        }
    }
}

impl MockStaking {
    pub fn new(infos: &[(&String, StakerResponse)]) -> Self {
        MockStaking {
            infos: infos_to_map(infos),
        }
    }
}

pub fn infos_to_map(infos: &[(&String, StakerResponse)]) -> HashMap<String, StakerResponse> {
    let mut info_map: HashMap<String, StakerResponse> = HashMap::new();
    for (staker, staker_info) in infos.iter() {
        info_map.insert(staker.to_string(), staker_info.clone());
    }
    info_map
}
