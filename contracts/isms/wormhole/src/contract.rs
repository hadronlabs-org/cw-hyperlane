use crate::helpers::new_event;
use crate::state::{Config, CONFIG, VERIFIED_IDS};
use crate::wormhole::{ParsedVAA, WormholeQueryMsg};
use crate::{ContractError, CONTRACT_NAME, CONTRACT_VERSION, WORMHOLE_CORE};
use cosmwasm_std::{
    ensure_eq, Binary, Deps, DepsMut, Empty, Env, HexBinary, MessageInfo, QueryResponse, Response,
};
use cw2::set_contract_version;
use hpl_interface::ism::wormhole::{ExecuteMsg, InstantiateMsg, QueryMsg, WormholeIsmQueryMsg};
use hpl_interface::ism::IsmQueryMsg::{ModuleType, Verify, VerifyInfo};
use hpl_interface::ism::{IsmType, ModuleTypeResponse, VerifyInfoResponse, VerifyResponse};
use hpl_interface::to_binary;
use hpl_interface::types::Message;

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

    CONFIG.save(
        deps.storage,
        &Config {
            emitter_chain: msg.emitter_chain,
            emitter_address: msg.emitter_address,
            origin_domain: msg.origin_domain,
            origin_sender: msg.origin_sender,
        },
    )?;

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
            handle_set_wormhole_core(deps, info, wormhole_core)
        }
        // metadata is actually wormhole VAA data in order for it to work
        ExecuteMsg::SubmitMeta { metadata, message } => handle_submit_meta(deps, metadata, message),
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

fn handle_set_wormhole_core(
    deps: DepsMut,
    info: MessageInfo,
    wormhole_core: String,
) -> Result<Response, ContractError> {
    ensure_eq!(
        hpl_ownable::get_owner(deps.storage)?,
        info.sender,
        ContractError::Unauthorized
    );

    let wormhole_core = deps.api.addr_validate(&wormhole_core)?;
    WORMHOLE_CORE.save(deps.storage, &wormhole_core)?;

    Ok(Response::new()
        .add_event(new_event("set_wormhole_core").add_attribute("wormhole_core", wormhole_core)))
}

fn handle_submit_meta(
    deps: DepsMut,
    metadata: HexBinary,
    message: HexBinary,
) -> Result<Response, ContractError> {
    // unpack and verify vaa and check that the message is indeed (indeed what?)
    let packed_id = unpack_verify_vaa(deps.as_ref(), metadata, message)?;

    VERIFIED_IDS.save(deps.storage, packed_id.clone().into(), &())?;

    Ok(Response::default()
        .add_event(new_event("submit_meta"))
        .add_attribute("packed_id", packed_id.to_hex()))
}

/// **unpack_verify_vaa** uses core wormhole contract to verify and unpack the vaa inside metadata
/// It also compares it to the message id.
/// Also verify that that origin sender and origin chain is as expected.
fn unpack_verify_vaa(
    deps: Deps,
    metadata: HexBinary,
    message: HexBinary,
) -> Result<HexBinary, ContractError> {
    let wormhole_core = WORMHOLE_CORE.load(deps.storage)?;
    let wormhole_query_msg = WormholeQueryMsg::VerifyVAA {
        vaa: Binary::from(metadata.as_slice()),
        block_time: 0,
    };
    let parsed_vaa: ParsedVAA = deps
        .querier
        .query_wasm_smart(wormhole_core, &wormhole_query_msg)?;

    let packed_id = HexBinary::from(parsed_vaa.payload.clone());

    let message: Message = message.into();
    let id = message.id();

    ensure_eq!(id, packed_id, ContractError::IdsDontMatch);

    let config = CONFIG.load(deps.storage)?;
    ensure_eq!(
        parsed_vaa.emitter_chain,
        config.emitter_chain,
        ContractError::OriginDoesNotMatch
    );
    ensure_eq!(
        parsed_vaa.emitter_address,
        config.emitter_address,
        ContractError::OriginDoesNotMatch
    );
    ensure_eq!(
        message.origin_domain,
        config.origin_domain,
        ContractError::OriginDoesNotMatch
    );
    ensure_eq!(
        message.sender,
        config.origin_sender,
        ContractError::OriginDoesNotMatch
    );

    Ok(packed_id)
}

fn verify(
    deps: Deps,
    metadata: HexBinary,
    message: HexBinary,
) -> Result<VerifyResponse, ContractError> {
    // 1. verify that the message is indeed passed the check (unnecessary since the message.id is unique anyway?)
    let packed_id = unpack_verify_vaa(deps, metadata, message)?;

    // 2. check the map
    let verified = VERIFIED_IDS.has(deps.storage, packed_id.into());

    Ok(VerifyResponse { verified })
}

// TODO: what is this for?
// TODO: implement
fn verify_info(deps: Deps, _message: HexBinary) -> Result<VerifyInfoResponse, ContractError> {
    // this is not entirely correct, but I don't see a better way to do this
    // we cannot query validators from Wormhole Core contract
    Ok(VerifyInfoResponse {
        threshold: 1,
        validators: vec![],
    })
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    Ok(Response::default())
}
