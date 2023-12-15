use super::IsmQueryMsg;
use crate::ownable::{OwnableMsg, OwnableQueryMsg};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::HexBinary;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub wormhole_core: String,

    // TODO: comments
    pub vaa_emitter_chain: u16,
    pub vaa_emitter_address: HexBinary,
    pub hyperlane_origin_domain: u32,
    pub hyperlane_origin_sender: HexBinary,
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
    /// We can check that this message id was marked as verified in the `Verify` query
    SubmitMeta {
        /// **vaa** is the wormhole vaa message packed into hex binary
        vaa: HexBinary,
        /// **message** is the hyperlane message packed into hex binary
        message: HexBinary,
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
