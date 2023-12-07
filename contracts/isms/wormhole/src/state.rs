use cosmwasm_std::HexBinary;
use cw_storage_plus::Map;

/// **VERIFIED_IDS** contains all the `message.id`s (hash) that were verified by core wormhole contract
pub const VERIFIED_IDS: Map<Vec<u8>, ()> = Map::new("verified-ids");
