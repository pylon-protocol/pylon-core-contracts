use cosmwasm_std::StdResult;

pub trait Validator {
    fn validate(&self) -> StdResult<()>;
}
