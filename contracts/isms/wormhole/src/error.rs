use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("ids of message and vaa data does not match")]
    IdsDontMatch,

    #[error("message id is not verified")]
    IdIsNotVerified,

    #[error("origin data does not match")]
    OriginDoesNotMatch,
}
