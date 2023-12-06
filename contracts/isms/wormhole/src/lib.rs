pub mod error;

#[cfg(test)]
pub mod tests;
mod contract;
mod wormhole;
mod helpers;

pub use crate::error::ContractError;
use cosmwasm_std::Addr;
use cw_storage_plus::Item;

// version info for migration info
pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const WORMHOLE_CORE_KEY: &str = "wormhole_core";
const WORMHOLE_CORE: Item<Addr> = Item::new(WORMHOLE_CORE_KEY);
