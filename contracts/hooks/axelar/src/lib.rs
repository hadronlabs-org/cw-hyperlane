#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, ensure_eq, Addr, Deps, DepsMut, Env, Event, HexBinary, MessageInfo, QueryResponse,
    Response, StdError, StdResult, Uint128,
};
use cw_storage_plus::Item;
use ethabi::{encode, Token};
use hpl_interface::{
    core::mailbox::{LatestDispatchedIdResponse, MailboxQueryMsg},
    hook::{
        axelar::{
            AxelarGeneralMessage, AxelarInfoResponse, AxelarQueryMsg, ExecuteMsg, InstantiateMsg,
            QueryMsg, RegisterDestinationContractMsg,
        },
        HookQueryMsg, MailboxResponse, PostDispatchMsg, QuoteDispatchMsg, QuoteDispatchResponse,
    },
    to_binary,
    types::{AxelarMetadata, Message},
};
use hpl_ownable::get_owner;
use neutron_sdk::{
    bindings::{
        msg::{IbcFee, NeutronMsg},
        query::NeutronQuery,
    },
    query::min_ibc_fee::query_min_ibc_fee,
    sudo::msg::RequestPacketTimeoutHeight,
};
use serde_json_wasm::to_string;

// version info for migration info
pub const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const FEE_DENOM: &str = "untrn";

const AXELAR_GATEWAY: &str = "axelar1dv4u5k73pzqrxlzujxg3qp8kvc3pje7jtdvu72npnt5zhq05ejcsn5qme5";

// TODO: move these to a single struct
const DESTINATION_CHAIN_KEY: &str = "destination_chain";
const DESTINATION_CHAIN: Item<String> = Item::new(DESTINATION_CHAIN_KEY);

const DESTINATION_CONTRACT_KEY: &str = "destination_contract";
const DESTINATION_CONTRACT: Item<String> = Item::new(DESTINATION_CONTRACT_KEY);

const DESTINATION_ISM_KEY: &str = "destination_ism";
const DESTINATION_ISM: Item<String> = Item::new(DESTINATION_ISM_KEY);

const AXELAR_GATEWAY_CHANNEL_KEY: &str = "axelar_gateway_channel";
const AXELAR_GATEWAY_CHANNEL: Item<String> = Item::new(AXELAR_GATEWAY_CHANNEL_KEY);

pub const MAILBOX_KEY: &str = "mailbox";
pub const MAILBOX: Item<Addr> = Item::new(MAILBOX_KEY);

pub const GAS_TOKEN_KEY: &str = "gas_token";
pub const GAS_TOKEN: Item<String> = Item::new(GAS_TOKEN_KEY);

fn new_event(name: &str) -> Event {
    Event::new(format!("hpl_hook_axelar::{}", name))
}

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    PaymentError(#[from] cw_utils::PaymentError),

    #[error("unauthorized")]
    Unauthorized {},

    #[error("hook paused")]
    Paused {},

    #[error("invalid recipient address")]
    InvalidRecipientAddress { address: String },

    #[error("last_dispatch query failed ")]
    LastDispatchQueryFailed { err: String },

    #[error("last_dispatch id mismatch")]
    LastDispatchIDMismatch { got: HexBinary, expected: HexBinary },
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let owner = deps.api.addr_validate(&msg.owner)?;
    hpl_ownable::initialize(deps.storage, &owner)?;

    let destination_chain = &msg.destination_chain;
    let destination_contract = &msg.destination_contract;
    let destination_ism = &msg.destination_ism;
    let axelar_gateway_channel = &msg.axelar_gateway_channel;
    let gas_token = &msg.gas_token;
    let mailbox: Addr = deps.api.addr_validate(&msg.mailbox)?;

    DESTINATION_CHAIN.save(deps.storage, destination_chain)?;
    DESTINATION_CONTRACT.save(deps.storage, destination_contract)?;
    DESTINATION_ISM.save(deps.storage, destination_ism)?;
    AXELAR_GATEWAY_CHANNEL.save(deps.storage, axelar_gateway_channel)?;
    GAS_TOKEN.save(deps.storage, gas_token)?;
    MAILBOX.save(deps.storage, &mailbox)?;

    Ok(Response::new().add_event(
        new_event("initialize")
            .add_attribute("sender", info.sender)
            .add_attribute("owner", owner)
            .add_attribute("destination_chain", destination_chain)
            .add_attribute("destination_contract", destination_contract)
            .add_attribute("destination_ism", destination_ism)
            .add_attribute("axelar_gateway_channel", axelar_gateway_channel),
    ))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    match msg {
        // TODO: maybe add SetWormholeCore Msg
        ExecuteMsg::Ownable(msg) => match hpl_ownable::handle(deps, env, info, msg) {
            Ok(s) => Ok(Response::new().add_events(s.events)),
            Err(e) => Err(e.into()),
        },
        ExecuteMsg::PostDispatch(msg) => post_dispatch(deps, env, info, msg),
        ExecuteMsg::RegisterDestinationContract(msg) => {
            register_destination_contract(deps, info, msg)
        }
    }
}

fn register_destination_contract(
    deps: DepsMut<NeutronQuery>,
    info: MessageInfo,
    msg: RegisterDestinationContractMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    ensure_eq!(
        get_owner(deps.storage)?,
        info.sender,
        ContractError::Unauthorized {}
    );

    let destination_contract = &msg.destination_contract;
    DESTINATION_CONTRACT.save(deps.storage, destination_contract)?;

    Ok(Response::new().add_event(
        new_event("register_destination_contract")
            .add_attribute("destination_contract", destination_contract),
    ))
}

fn post_dispatch(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    req: PostDispatchMsg,
) -> Result<Response<NeutronMsg>, ContractError> {
    // Ensure message_id matches latest dispatch from mailbox
    let mailbox = MAILBOX.load(deps.storage)?;
    let latest_dispatch_resp = deps
        .querier
        .query_wasm_smart::<LatestDispatchedIdResponse>(
            &mailbox,
            &MailboxQueryMsg::LatestDispatchId {}.wrap(),
        )
        .or_else(|err| {
            return Err(ContractError::LastDispatchQueryFailed {
                err: err.to_string(),
            });
        });

    let latest_dispatch_id = latest_dispatch_resp.unwrap().message_id;

    let decoded_msg: Message = req.message.clone().into();

    ensure_eq!(
        latest_dispatch_id,
        decoded_msg.id(),
        ContractError::LastDispatchIDMismatch {
            got: decoded_msg.id(),
            expected: latest_dispatch_id
        }
    );

    //send message to axelar gateway
    let destination_chain = DESTINATION_CHAIN.load(deps.storage)?;
    let destination_contract = DESTINATION_CONTRACT.load(deps.storage)?;
    let destination_ism = DESTINATION_ISM.load(deps.storage)?;
    let axelar_gateway_channel = AXELAR_GATEWAY_CHANNEL.load(deps.storage)?;

    // TODO: do we need to pass a fee?
    send_to_evm(
        deps,
        env,
        info,
        req,
        axelar_gateway_channel,
        destination_chain,
        destination_contract,
        vec![destination_ism],
    )
}

pub fn send_to_evm(
    deps: DepsMut<NeutronQuery>,
    env: Env,
    info: MessageInfo,
    req: PostDispatchMsg,
    gateway_channel: String,
    destination_chain: String,
    destination_contract: String,
    _destination_recipients: Vec<String>,
) -> Result<Response<NeutronMsg>, ContractError> {
    // let addresses = destination_recipients
    // .into_iter()
    // .map(|s| {
    //     match s.parse::<H160>() {
    //         Ok(address) => Ok(Token::Address(Address::from(address))),
    //         Err(_) => Err(ContractError::InvalidRecipientAddress { address: s }),
    //     }
    // })
    // .collect::<Result<Vec<Token>, ContractError>>()?;

    let message_nonce = Message::from(req.message).nonce;
    let message_payload = encode(&vec![
        Token::String(info.sender.to_string()),
        Token::Int(message_nonce.into()),
    ]);

    let mut destination_address: String = "0x".to_string();
    destination_address.push_str(&destination_contract);

    let msg = AxelarGeneralMessage {
        destination_chain,
        destination_address,
        payload: message_payload.to_vec(),
        type_: 1,
        // TODO: confirm there is no GMP fee
        fee: None,
    };

    let decoded_metadata: AxelarMetadata = req.metadata.clone().into();
    let axelar_fee_amt = decoded_metadata.gas_amount;
    let axelar_fee_coin = coin(axelar_fee_amt, GAS_TOKEN.load(deps.storage)?);

    let ibc_fee = min_ntrn_ibc_fee(query_min_ibc_fee(deps.as_ref()).unwrap().min_fee);

    let ibc_transfer = NeutronMsg::IbcTransfer {
        source_port: "transfer".to_string(),
        source_channel: gateway_channel,
        token: axelar_fee_coin,
        sender: env.contract.address.to_string(),
        receiver: AXELAR_GATEWAY.to_string(),
        timeout_height: RequestPacketTimeoutHeight {
            revision_number: None,
            revision_height: None,
        },
        timeout_timestamp: env.block.time.plus_seconds(604_800u64).nanos(),
        memo: to_string(&msg).unwrap(),
        fee: ibc_fee,
    };

    // Base response
    let response = Response::default()
        .add_attribute("status", "ibc_message_created")
        .add_attribute("ibc_message", format!("{:?}", ibc_transfer));
    let r = response.add_message(ibc_transfer);
    return Ok(r);
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps<NeutronQuery>,
    env: Env,
    msg: QueryMsg,
) -> Result<QueryResponse, ContractError> {
    deps.api.debug("inside top query");
    match msg {
        QueryMsg::Axelar(msg) => Ok(handle_query(deps, env, msg)?),
        QueryMsg::Ownable(msg) => Ok(hpl_ownable::handle_query(deps, env, msg)?),
        QueryMsg::Hook(msg) => match msg {
            HookQueryMsg::Mailbox {} => to_binary(get_mailbox()),
            HookQueryMsg::QuoteDispatch(msg) => to_binary(quote_dispatch(deps, msg)),
        },
    }
}

pub fn handle_query(
    deps: Deps<NeutronQuery>,
    _env: Env,
    _msg: AxelarQueryMsg,
) -> StdResult<QueryResponse> {
    cosmwasm_std::to_binary(&AxelarInfoResponse {
        destination_chain: DESTINATION_CHAIN.load(deps.storage)?,
        destination_contract: DESTINATION_CONTRACT.load(deps.storage)?,
        destination_ism: DESTINATION_ISM.load(deps.storage)?,
        axelar_gateway_channel: AXELAR_GATEWAY_CHANNEL.load(deps.storage)?,
    })
}

fn get_mailbox() -> Result<MailboxResponse, ContractError> {
    Ok(MailboxResponse {
        mailbox: "unrestricted".to_string(),
    })
}

fn quote_dispatch(
    deps: Deps<NeutronQuery>,
    msg: QuoteDispatchMsg,
) -> Result<QuoteDispatchResponse, ContractError> {
    // TODO: gaurd against casting and overflow issues
    let decoded_metadata: AxelarMetadata = msg.metadata.clone().into();
    let axelar_fee: Uint128 = decoded_metadata.gas_amount.into();
    let ibc_fees = min_ntrn_ibc_fee(query_min_ibc_fee(deps).unwrap().min_fee);
    let fee_total = axelar_fee + ibc_fees.ack_fee[0].amount + ibc_fees.timeout_fee[0].amount;

    // TODO: add better check to make sure the right metadata is present
    Ok(QuoteDispatchResponse {
        gas_amount: Some(coin(fee_total.into(), GAS_TOKEN.load(deps.storage)?)),
    })
}

fn min_ntrn_ibc_fee(fee: IbcFee) -> IbcFee {
    IbcFee {
        recv_fee: fee.recv_fee,
        ack_fee: fee
            .ack_fee
            .into_iter()
            .filter(|a| a.denom == FEE_DENOM)
            .collect(),
        timeout_fee: fee
            .timeout_fee
            .into_iter()
            .filter(|a| a.denom == FEE_DENOM)
            .collect(),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use cosmwasm_std::{
        from_binary,
        testing::{mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage},
        HexBinary, OwnedDeps, WasmQuery,
    };

    use hpl_interface::{
        build_test_executor, build_test_querier, core::mailbox, hook::QuoteDispatchMsg,
    };
    use hpl_ownable::get_owner;
    use ibcx_test_utils::hex;
    use rstest::{fixture, rstest};

    use crate::{execute, instantiate};

    type TestDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    build_test_executor!(self::execute);
    build_test_querier!(self::query);

    #[fixture]
    fn deps(
        #[default(Addr::unchecked("deployer"))] sender: Addr,
        #[default(Addr::unchecked("owner"))] owner: Addr,
        #[default(Addr::unchecked("mailbox"))] mailbox: Addr,
    ) -> TestDeps {
        let mut deps = mock_dependencies();

        instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(sender.as_str(), &[]),
            InstantiateMsg {
                owner: owner.to_string(),
                mailbox: mailbox.to_string(),
                destination_chain: "test-chain".to_string(),
                destination_contract: "test_contract".to_string(),
                gas_token: "untrn".to_string(),
                axelar_gateway_channel: "channel-1".to_string(),
                destination_ism: "4D147dCb984e6affEEC47e44293DA442580A3Ec0".to_string(),
            },
        )
        .unwrap();

        deps
    }

    // #[rstest]
    // fn test_post_dispatch(){
    //     let message = Message{
    //         version: 1,
    //         sender: HexBinary::from_hex("6E657574726F6E317877747A397733706664336E68373765326C76656C7A7A3064773768783372786E6D7A746573").unwrap(),
    //         nonce: 2,
    //         origin_domain: 10,
    //         recipient: HexBinary::from_hex("6E657574726F6E317877747A397733706664336E68373765326C76656C7A7A3064773768783372786E6D7A746573").unwrap(),
    //         body: HexBinary::from_hex("686920776F726C64").unwrap(),
    //         dest_domain: 20,
    //     };

    //     let pd_message = PostDispatchMsg{
    //         metadata: HexBinary::default(),
    //         message: message.into(),
    //     };
    //     dispatch_resp = post_dispatch(deps.as_mut(), env, info, req)
    // }

    #[rstest]
    fn test_quote_dispatch() {
        let metadata: HexBinary = AxelarMetadata { gas_amount: 200 }.into();
        print!("{}", metadata.to_hex());
        let quote_dispatch_msg = QuoteDispatchMsg {
            metadata,
            message: HexBinary::from_hex("68656C6C6F").unwrap(),
        };
        let decoded_metadata: AxelarMetadata = quote_dispatch_msg.metadata.clone().into();
        assert_eq!(decoded_metadata.gas_amount, 200)
    }
}
