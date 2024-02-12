use super::IsmQueryMsg;
use crate::ownable::{OwnableMsg, OwnableQueryMsg};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::HexBinary;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub wormhole_core: String,

    pub vaa_emitter_chain: u16,
    pub hyperlane_origin_domain: u32,
}

#[cw_serde]
pub enum ExecuteMsg {
    Ownable(OwnableMsg),

    /// **SetWormholeCore** sets the wormhole core contract address that we use to verify VAA message
    SetWormholeCore {
        wormhole_core: String,
    },

    /// **SetOriginAddress** sets the origin EVM address that we check for wormhole VAA message and for hyperlane message
    SetOriginAddress {
        address: Vec<u8>,
    },

    /// **SubmitMeta** step is called by the wormhole relayer.
    /// We verify the metadata and compare it to the message id
    /// Then as we're sure that this metadata with message is legit,
    /// We can check that this message id was marked as verified in the `Verify` query
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
