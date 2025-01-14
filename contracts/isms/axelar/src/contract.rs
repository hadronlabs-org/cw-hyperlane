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
use hpl_ownable::get_owner;

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
        ExecuteMsg::SetOriginAddress { origin_address } => {
            handle_set_origin_address(deps, info, origin_address)
        }
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
                    typ: IsmType::Null,
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
    origin_address: String,
) -> Result<Response, ContractError> {
    ensure_eq!(
        get_owner(deps.storage)?,
        info.sender,
        ContractError::Unauthorized {
            expected: "owner".to_string()
        }
    );

    let mut config = CONFIG.load(deps.storage)?;
    config.origin_address = Some(origin_address.clone());
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_event(new_event("set_origin_address").add_attribute("orgin_address", origin_address)))
}
// TODO
fn handle_submit_meta(
    deps: DepsMut,
    _info: MessageInfo,
    origin_address: String, // TODO: naming
    origin_chain: String,   // TODO: naming
    id: Vec<u8>,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let id_hex_binary = HexBinary::from(id);
    // TODO: confirm we don't need this
    // ensure_eq!(
    //     config.axelar_hook_sender,
    //     info.sender.to_string(),
    //     ContractError::Unauthorized {expected: info.sender.to_string()}
    // );

    let config_origin_address = config
        .origin_address
        .ok_or_else(|| ContractError::OriginAddressNotSet)?;
    ensure_eq!(
        config_origin_address.to_lowercase(),
        origin_address.to_lowercase(),
        ContractError::InvalidOriginAddress {
            expected: origin_address
        }
    );

    ensure_eq!(
        config.origin_chain.to_lowercase(),
        origin_chain.to_lowercase(),
        ContractError::InvalidOriginChain {
            expected: origin_chain
        }
    );

    VERIFIED_IDS.save(deps.storage, id_hex_binary.to_string(), &())?;

    Ok(Response::default().add_event(
        new_event("submit_meta")
            .add_attribute("ID", id_hex_binary.to_string())

    ))
}

fn verify(
    deps: Deps,
    _metadata: HexBinary,
    message: HexBinary,
) -> Result<VerifyResponse, ContractError> {
    let message: Message = message.into();
    let verified = VERIFIED_IDS.has(deps.storage, message.id().to_string());
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

#[cfg(test)]
mod tests {
    use hpl_interface::types::Message;
    use cosmwasm_std::{
        ensure_eq, Deps, DepsMut, Empty, Env, HexBinary, MessageInfo, QueryResponse, Response,
    };

    #[test]
    fn message_hex_test(){
        let hex = |v: &str| -> HexBinary { HexBinary::from_hex(v).unwrap() };

        let message = Message{

             version: 3,
             nonce: 5556,
             origin_domain: 11155111,
             sender: hex("0000000000000000000000008014ea96f219a59c183cec9794039bbe3167a847"),
             dest_domain: 33333,
             recipient: hex("ef674da84b7e89fdc57a31a05c7dd432a89df6eba869f833adf51d623c5d3554"),
             body: hex("000000000000000000000000b352b279f34234ca9a457cab9b20c4580013dadd000000000000000000000000000000000000000000000000002386f26fc10000")
        };
        println!("ID: {:?}", message.id());
        println!("HB: {:?}", HexBinary::from(message));
        let sub_id = [
            31,
            68,
            167,
            170,
            162,
            109,
            64,
            204,
            62,
            109,
            73,
            163,
            15,
            191,
            14,
            62,
            233,
            216,
            243,
            96,
            222,
            89,
            2,
            5,
            186,
            234,
            189,
            251,
            71,
            203,
            222,
            11
          ];
          let sub_hex = HexBinary::from(sub_id);
          println!("sub hex: {:?}", sub_hex);
    }

    // expected ID: "0x1f44a7aaa26d40cc3e6d49a30fbf0e3ee9d8f360de590205baeabdfb47cbde0b"),
}
