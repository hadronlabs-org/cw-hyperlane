use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized")]
    Unauthorized {expected: String},

    #[error("invalid origin address")]
    InvalidOriginAddress {expected: String},

    #[error("invalid origin chain")]
    InvalidOriginChain {expected: String},
}
