use cosmwasm_std::{Binary, Deps, DepsMut, Empty, ensure_eq, Env, HexBinary, MessageInfo, QueryResponse, Response};
use cw2::set_contract_version;
use hpl_interface::ism::IsmQueryMsg::{ModuleType, Verify, VerifyInfo};
use hpl_interface::ism::{IsmType, ModuleTypeResponse, VerifyInfoResponse, VerifyResponse};
use hpl_interface::ism::wormhole::{ExecuteMsg, InstantiateMsg, QueryMsg, WormholeIsmQueryMsg};
use hpl_interface::to_binary;
use hpl_interface::types::{bech32_decode, Message};
use crate::{CONTRACT_NAME, CONTRACT_VERSION, ContractError, WORMHOLE_CORE};
use crate::ContractError::Unauthorized;
use crate::helpers::new_event;
use crate::wormhole::{ParsedVAA, WormholeQueryMsg};

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    hpl_ownable::initialize(deps.storage, &owner)?;

    let wormhole_core = deps.api.addr_validate(&msg.wormhole_core)?;
    WORMHOLE_CORE.save(deps.storage, &wormhole_core)?;

    Ok(Response::new().add_event(
        new_event("instantiate")
            .add_attribute("sender", info.sender)
            .add_attribute("owner", owner)
            .add_attribute("wormhole_core", wormhole_core),
    ))
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Ownable(msg) => Ok(hpl_ownable::handle(deps, env, info, msg)?),
        ExecuteMsg::SetWormholeCore { wormhole_core } => {
            ensure_eq!(
                hpl_ownable::get_owner(deps.storage)?,
                info.sender,
                ContractError::Unauthorized
            );

            let wormhole_core = deps.api.addr_validate(&wormhole_core)?;
            WORMHOLE_CORE.save(deps.storage, &wormhole_core)?;

            Ok(Response::new().add_event(
                new_event("set_wormhole_core").add_attribute("wormhole_core", wormhole_core),
            ))
        }

        ExecuteMsg::SubmitVAA { metadata, message } => {
            id = get_from_cbc(metadata);
            map[id] = true
        }
    }
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<QueryResponse, ContractError> {
    match msg {
        QueryMsg::Ownable(msg) => Ok(hpl_ownable::handle_query(deps, env, msg)?),

        QueryMsg::Ism(msg) => match msg {
            ModuleType {} => to_binary({
                Ok::<_, ContractError>(ModuleTypeResponse {
                    typ: IsmType::Wormhole,
                })
            }),
            Verify { metadata, message } => to_binary(verify(deps, metadata, message)),
            VerifyInfo { message } => to_binary(verify_info(deps, message)),
        },

        QueryMsg::WormholeIsm(msg) => match msg {
            WormholeIsmQueryMsg::WormholeCore {} => Ok(cosmwasm_std::to_binary(
                &WORMHOLE_CORE.load(deps.storage)?.into_string(),
            )?),
        },
    }
}

fn verify(
    deps: Deps,
    metadata: HexBinary,
    message: HexBinary,
) -> Result<VerifyResponse, ContractError> {
    let wormhole_core = WORMHOLE_CORE.load(deps.storage)?;
    let wormhole_query_msg = WormholeQueryMsg::VerifyVAA {
        vaa: Binary::from(metadata.as_slice()),
        block_time: 0,
    };

    let parsed_vaa: ParsedVAA = deps
        .querier
        .query_wasm_smart(wormhole_core, &wormhole_query_msg)?;

    let message: Message = message.into();
    let id = message.id();

    Ok(VerifyResponse {
        verified: HexBinary::from(parsed_vaa.payload) == id,
        // verified: map[id]
    })
}

fn verify_info(deps: Deps, _message: HexBinary) -> Result<VerifyInfoResponse, ContractError> {
    // this is not entirely correct, but I don't see a better way to do this
    // we cannot query validators from Wormhole Core contract
    Ok(VerifyInfoResponse {
        threshold: 1,
        validators: vec![bech32_decode(WORMHOLE_CORE.load(deps.storage)?.as_str())?.into()],
    })
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    Ok(Response::default())
}
