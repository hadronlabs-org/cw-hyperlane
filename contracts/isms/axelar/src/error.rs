use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("origin address not set")]
    OriginAddressNotSet,

    #[error("invalid origin address")]
    InvalidOriginAddress,

    #[error("invalid origin chain")]
    InvalidOriginChain,
}
