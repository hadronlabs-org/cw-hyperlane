use cosmwasm_schema::cw_serde;
use cosmwasm_std::HexBinary;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    // TODO: comments
    /// for wormhole verification
    pub vaa_emitter_chain: u16,
    pub vaa_emitter_address: HexBinary,

    /// for hyperlane message verification
    pub hyperlane_origin_domain: u32,
    pub hyperlane_origin_sender: HexBinary,
}

/// **VERIFIED_IDS** contains all the `message.id`s (hash) that were verified by core wormhole contract
pub const VERIFIED_IDS: Map<Vec<u8>, ()> = Map::new("verified_ids");

/// **CONFIG** is the contract's config
pub const CONFIG: Item<Config> = Item::new("config");
