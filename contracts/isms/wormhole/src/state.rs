use cosmwasm_schema::cw_serde;
use cosmwasm_std::HexBinary;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    // TODO: comments
    pub emitter_chain: u16,
    pub emitter_address: Vec<u8>,
    pub origin_domain: u32,
    pub origin_sender: Vec<u8>,
}

/// **VERIFIED_IDS** contains all the `message.id`s (hash) that were verified by core wormhole contract
pub const VERIFIED_IDS: Map<Vec<u8>, ()> = Map::new("verified-ids");

/// **CONFIG** is the contract's config
pub const CONFIG: Item<Config> = Item::new("config");
