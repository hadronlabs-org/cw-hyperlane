use crate::helpers::new_event;
use crate::state::{Config, CONFIG, VERIFIED_IDS};
use crate::{ContractError, CONTRACT_NAME, CONTRACT_VERSION};
use cosmwasm_std::{
    ensure_eq, Deps, DepsMut, Empty, Env, HexBinary, MessageInfo, QueryResponse, Response,
};
use cw2::set_contract_version;
use hpl_interface::ism::axelar::{ExecuteMsg, InstantiateMsg, QueryMsg};
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

    let config = Config {
        axelar_hook_sender: msg.axelar_hook_sender,
        origin_chain: msg.origin_chain,
        origin_address: None,
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().add_event(
        new_event("instantiate")
            .add_attribute("sender", info.sender)
            .add_attribute("owner", owner),
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
        ExecuteMsg::SetOriginAddress { address } => handle_set_origin_address(deps, info, address),
        // metadata is actually VAA data in order for it to work
        ExecuteMsg::SubmitMeta {
            origin_address,
            origin_chain,
            id,
        } => handle_submit_meta(deps, info, origin_address, origin_chain, id),
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
    }
}

fn handle_set_origin_address(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    ensure_eq!(
        hpl_ownable::get_owner(deps.storage)?,
        info.sender,
        ContractError::Unauthorized
    );

    let mut config = CONFIG.load(deps.storage)?;
    config.origin_address = Some(address);
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

fn handle_submit_meta(
    deps: DepsMut,
    info: MessageInfo,
    origin_address: String,
    origin_chain: String,
    id: HexBinary,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    ensure_eq!(
        config.axelar_hook_sender,
        info.sender,
        ContractError::Unauthorized
    );

    let config_origin_address = config
        .origin_address
        .ok_or_else(|| ContractError::OriginAddressNotSet)?;
    ensure_eq!(
        config_origin_address,
        origin_address,
        ContractError::InvalidOriginAddress
    );

    ensure_eq!(
        config.origin_chain,
        origin_chain,
        ContractError::InvalidOriginChain
    );

    // TODO: verify recipient?

    VERIFIED_IDS.save(deps.storage, id.to_vec(), &())?;

    Ok(Response::default().add_event(new_event("")))
}

fn verify(
    deps: Deps,
    _metadata: HexBinary,
    message: HexBinary,
) -> Result<VerifyResponse, ContractError> {
    let message: Message = message.into();
    let verified = VERIFIED_IDS.has(deps.storage, message.id().into());
    Ok(VerifyResponse { verified })
}

// TODO: what is this for?
// TODO: implement
fn verify_info(deps: Deps, _message: HexBinary) -> Result<VerifyInfoResponse, ContractError> {
    Ok(VerifyInfoResponse {
        threshold: 1,
        validators: vec![],
    })
}

#[cfg_attr(not(feature = "library"), cosmwasm_std::entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    Ok(Response::default())
}
