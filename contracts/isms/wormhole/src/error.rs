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

    #[error("vaa emitter chain does not match {vaa} != {config}")]
    VaaEmitterChainDoesNotMatch { vaa: u16, config: u16 },

    #[error("vaa emitter address does not match")]
    VaaEmitterAddressDoesNotMatch,

    #[error("message origin domain does not match {message} != {config}")]
    MessageOriginDomainDoesNotMatch  { message: u16, config: u16 },

    #[error("message origin sender does not match {message} != {config}")]
    MessageOriginSenderDoesNotMatch { message: String, config: String },
}
