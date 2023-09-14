use cosmwasm_std::{to_binary, DepsMut, Env, HexBinary, MessageInfo, Response, WasmMsg};
use hpl_interface::types::message::Message;

use crate::{event::emit_post_dispatch, state::HOOK_CONFIG, ContractError};

pub fn dispatch(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    metadata: HexBinary,
    message: HexBinary,
) -> Result<Response, ContractError> {
    let hpl_msg: Message = message.clone().into();
    let target_contract = HOOK_CONFIG
        .load(deps.storage, hpl_msg.dest_domain)
        .map_err(|_| ContractError::HookNotRegistered(hpl_msg.dest_domain))?;

    let wasm_msg = WasmMsg::Execute {
        contract_addr: target_contract.hook.to_string(),
        msg: to_binary(
            &hpl_interface::post_dispatch_hook::PostDispatchMsg::PostDispatch {
                metadata: metadata.clone(),
                message: message.clone(),
            },
        )?,
        funds: vec![],
    };

    Ok(Response::new()
        .add_message(wasm_msg)
        .add_event(emit_post_dispatch(target_contract.hook, metadata, message)))
}