use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::HexBinary;

use crate::{
    core,
    ownable::OwnableQueryMsg,
    router::{self, RouterQuery},
};

use super::{TokenModeMsg, TokenWarpDefaultQueryMsg};

pub use cw20_base::msg::InstantiateMsg as Cw20InitMsg;

#[cw_serde]
pub enum TokenOption {
    Create {
        code_id: u64,
        init_msg: Box<cw20_base::msg::InstantiateMsg>,
    },
    Reuse {
        contract: String,
    },
}

#[cw_serde]
pub struct Cw20ModeBridged {
    pub code_id: u64,
    pub init_msg: Box<cw20_base::msg::InstantiateMsg>,
}

#[cw_serde]
pub struct Cw20ModeCollateral {
    pub address: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub token: TokenModeMsg<Cw20ModeBridged, Cw20ModeCollateral>,

    pub hrp: String,
    pub owner: String,
    pub mailbox: String,
}

#[cw_serde]
pub enum ReceiveMsg {
    // transfer to remote
    TransferRemote {
        dest_domain: u32,
        recipient: HexBinary,
    },
}

#[cw_serde]
pub enum ExecuteMsg {
    Router(router::RouterMsg<HexBinary>),

    /// handle transfer remote
    Handle(core::HandleMsg),

    // cw20 receiver
    Receive(cw20::Cw20ReceiveMsg),
}

#[cw_serde]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg {
    Ownable(OwnableQueryMsg),
    Router(RouterQuery<HexBinary>),
    TokenDefault(TokenWarpDefaultQueryMsg),
}