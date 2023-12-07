pub mod error;

mod axelar;
mod contract;
mod helpers;
mod state;
#[cfg(test)]
pub mod tests;

pub use crate::error::ContractError;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

// version info for migration info
pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
