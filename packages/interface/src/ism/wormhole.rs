use super::IsmQueryMsg;
use crate::ownable::{OwnableMsg, OwnableQueryMsg};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::HexBinary;

#[cw_serde]
pub struct InstantiateMsg {
    pub owner: String,
    pub wormhole_core: String,
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
        message: HexBinary,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg {
    Ownable(OwnableQueryMsg),

    Ism(IsmQueryMsg),

    WormholeIsm(WormholeIsmQueryMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum WormholeIsmQueryMsg {
    #[returns(String)]
    WormholeCore {},
}
