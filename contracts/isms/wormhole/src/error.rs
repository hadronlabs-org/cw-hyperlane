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

    #[error("origin address is not set")]
    OriginAddressNotSet,

    #[error("vaa emitter chain does not match {vaa} != {config}")]
    VaaEmitterChainDoesNotMatch { vaa: u16, config: u16 },

    #[error("vaa emitter address does not match {vaa} != {config}")]
    VaaEmitterAddressDoesNotMatch { vaa: String, config: String },
}
