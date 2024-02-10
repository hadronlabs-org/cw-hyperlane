use super::IsmQueryMsg;
use crate::ownable::{OwnableMsg, OwnableQueryMsg};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::HexBinary;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub wormhole_core: String,

    // TODO: comments
    pub emitter_chain: u16,
    pub emitter_address: Vec<u8>,
    pub origin_domain: u32,
    pub origin_sender: Vec<u8>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Ownable(OwnableMsg),

    SetWormholeCore {
        wormhole_core: String,
    },

    /// **SubmitMeta** step is called by the wormhole relayer.
    /// We verify the metadata and compare it to the message id
    /// Then as we're sure that this metadata with message is legit,
    /// We can check that this message id was passed in the `Verify` query
    SubmitMeta {
        metadata: HexBinary,
        // TODO: verify that this is not neccesary and remove
        // message: HexBinary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg {
    // TODO: what is this for?
    Ownable(OwnableQueryMsg),

    Ism(IsmQueryMsg),

    // TODO: what is this for?
    WormholeIsm(WormholeIsmQueryMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum WormholeIsmQueryMsg {
    #[returns(String)]
    WormholeCore {},
}
