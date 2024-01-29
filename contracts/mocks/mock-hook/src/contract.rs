use cosmwasm_schema::cw_serde;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Deps, DepsMut, Env, Event, MessageInfo, QueryResponse, Response, StdResult,
    Uint256,
};
use cw2::set_contract_version;
use cw_storage_plus::Item;
use hpl_interface::hook::{
    ExpectedHookQueryMsg, HookQueryMsg, PostDispatchMsg, QuoteDispatchResponse,
};

use crate::{CONTRACT_NAME, CONTRACT_VERSION};

const DEFAULT_GAS: u128 = 500_000;
const DEFAULT_GAS_TOKEN: &str = "untrn";

const GAS_KEY: &str = "gas";
const GAS: Item<Uint256> = Item::new(GAS_KEY);

const GAS_TOKEN_KEY: &str = "gas_token";
const GAS_TOKEN: Item<String> = Item::new(GAS_TOKEN_KEY);

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    SetGasAmount { gas: Uint256 },
    PostDispatch(PostDispatchMsg),
}

fn new_event(name: &str) -> Event {
    Event::new(format!("hpl_mock_hook::{name}"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    GAS.save(deps.storage, &Uint256::from_u128(DEFAULT_GAS))?;
    GAS_TOKEN.save(deps.storage, &DEFAULT_GAS_TOKEN.to_string())?;

    Ok(Response::new().add_event(new_event("instantiate")))
}

/// Handling contract execution
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::PostDispatch(PostDispatchMsg { metadata, message }) => Ok(Response::new()
            .add_event(
                new_event("post-dispatch")
                    .add_attribute("gas", GAS.load(deps.storage)?)
                    .add_attribute("sender", info.sender)
                    .add_attribute("message", message.to_string())
                    .add_attribute(
                        "metadata",
                        if metadata.is_empty() {
                            "0x".to_string()
                        } else {
                            metadata.to_string()
                        },
                    ),
            )),
        ExecuteMsg::SetGasAmount { gas } => {
            GAS.save(deps.storage, &gas)?;

            Ok(Response::new())
        }
    }
}

/// Handling contract query
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: ExpectedHookQueryMsg) -> StdResult<QueryResponse> {
    match msg {
        ExpectedHookQueryMsg::Hook(msg) => match msg {
            HookQueryMsg::QuoteDispatch(_) => {
                let gas = GAS.load(deps.storage)?;
                let gas_token = GAS_TOKEN.load(deps.storage)?;

                Ok(to_binary(&QuoteDispatchResponse {
                    gas_amount: Some(coin(gas.to_string().parse::<u128>().unwrap(), gas_token)),
                })?)
            }
            HookQueryMsg::Mailbox {} => {
                unimplemented!("mailbox query not implemented on mock hook")
            }
        },
    }
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{from_binary, to_binary, HexBinary};
    use hpl_interface::hook::{ExpectedHookMsg, PostDispatchMsg};

    use super::ExecuteMsg;

    #[test]
    fn test_schema() {
        // no need to test query - because it uses ExecptedHookQueryMsg directly!

        // test execute
        let _: ExecuteMsg = from_binary(
            &to_binary(&ExpectedHookMsg::PostDispatch(PostDispatchMsg {
                metadata: HexBinary::default(),
                message: HexBinary::default(),
            }))
            .unwrap(),
        )
        .unwrap();
    }
}
