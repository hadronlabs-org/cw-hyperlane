use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, HexBinary};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    /// **axelar_hook_sender** is a fixed address.
    /// Only this sender can execute `ExecuteMsg::SubmitMeta` message.
    /// This way we verify that this contract is really called through axelar using `ibc-hooks` module
    pub axelar_hook_sender: Addr,

    /// **origin_address** is an address.
    /// It represents expected origin address on EVM side
    pub origin_address: String,

    /// **origin_address** is a chain ID.
    /// It represents expected origin chain id on EVM side
    pub origin_chain: String,
}

/// **VERIFIED_IDS** contains all the `message.id`s (hash) that were verified by core wormhole contract.
/// Map key is the message id.
pub const VERIFIED_IDS: Map<String, ()> = Map::new("verified-ids");

/// **CONFIG** contains contract's configuration
pub const CONFIG: Item<Config> = Item::new("config");
