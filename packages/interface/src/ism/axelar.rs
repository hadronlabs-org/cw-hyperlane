use super::IsmQueryMsg;
use crate::ownable::{OwnableMsg, OwnableQueryMsg};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, HexBinary};

#[cw_serde]
pub struct InstantiateMsg {
    /// **owner** is a contract owner
    pub owner: String,

    /// **axelar_hook_sender** is an address.
    /// Only sender with this address can execute `ExecuteMsg::SubmitMeta` message.
    /// This way we verify that this contract is really called through axelar using `ibc-hooks` module
    pub axelar_hook_sender: Addr,

    /// **origin_address** is an address.
    /// It represents expected origin address on EVM side
    pub origin_address: String,

    /// **origin_address** is a chain ID.
    /// It represents expected origin chain id on EVM side
    pub origin_chain: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Ownable(OwnableMsg),

    /// **SubmitMeta** step is called by the wormhole relayer.
    /// We verify the metadata and compare it to the message id
    /// Then as we're sure that this metadata with message is legit,
    /// We can check that this message id was passed in the `Verify` query
    /// [permissioned - axelar_hook_sender only]
    SubmitMeta {
        origin_address: String,
        origin_chain: String,
        id: Vec<u8>,
    },
    SetOriginAddress {
        origin_address: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg {
    // TODO: what is this for?
    Ownable(OwnableQueryMsg),

    Ism(IsmQueryMsg),
}
