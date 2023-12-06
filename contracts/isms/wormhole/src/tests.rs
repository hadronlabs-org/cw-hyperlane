
#[cfg(test)]
mod tests {
    use crate::{verify, ParsedVAA, WormholeQueryMsg, WORMHOLE_CORE};
    use cosmwasm_std::{
        from_binary, from_slice,
        testing::{MockApi, MockStorage},
        to_binary, Addr, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError,
        SystemResult, WasmQuery,
    };
    use hpl_interface::ism::VerifyResponse;
    use ibcx_test_utils::hex;
    use std::marker::PhantomData;

    #[derive(Default)]
    struct CustomMockQuerier {}
    impl Querier for CustomMockQuerier {
        fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
            let request = match from_slice::<QueryRequest<Empty>>(bin_request).map_err(move |err| {
                QuerierResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", err),
                    request: bin_request.into(),
                })
            }) {
                Ok(v) => v,
                Err(e) => return e,
            };
            match request {
                QueryRequest::Wasm(request) => match request {
                    WasmQuery::Smart { contract_addr, msg } => {
                        const VAA: &str = "AQAAAAABAEvaVvB61VMTIBPWEgQstR04OEv9Stj+mZ2CkPwlPDRIfdL2MMXRViirkq0bHbUMtQM9gcAymhWj9NbT68PdER0BZV2uKAAAAAAAAgAAAAAAAAAAAAAAALlNEM3S65/plozvlT1fbhBX58ZAAAAAAAAAAADISIvbtb29zQ1v9P7OL15QdlONeG2fpn/7Ldsuf4RQ2ZA=";
                        assert_eq!(contract_addr, "wormhole_core");
                        let WormholeQueryMsg::VerifyVAA { vaa, block_time } =
                            from_binary(&msg).unwrap();
                        assert_eq!(block_time, 0);
                        assert_eq!(vaa.to_string(), VAA);
                        let vaa = ParsedVAA {
                            version: 1,
                            guardian_set_index: 0,
                            timestamp: 1700638248,
                            nonce: 0,
                            len_signers: 1,
                            emitter_chain: 2,
                            emitter_address: vec![
                                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 185, 77, 16, 205, 210, 235,
                                159, 233, 150, 140, 239, 149, 61, 95, 110, 16, 87, 231, 198, 64,
                            ],
                            sequence: 0,
                            consistency_level: 200,
                            payload: vec![
                                72, 139, 219, 181, 189, 189, 205, 13, 111, 244, 254, 206, 47, 94,
                                80, 118, 83, 141, 120, 109, 159, 166, 127, 251, 45, 219, 46, 127,
                                132, 80, 217, 144,
                            ],
                            hash: vec![
                                23, 195, 158, 108, 197, 235, 130, 102, 185, 255, 225, 41, 128, 71,
                                192, 121, 198, 19, 185, 49, 121, 235, 149, 124, 199, 132, 227, 245,
                                29, 120, 129, 140,
                            ],
                        };
                        SystemResult::Ok(cosmwasm_std::ContractResult::from(to_binary(&vaa)))
                    }
                    _ => unimplemented!(),
                },
                _ => unimplemented!(),
            }
        }
    }

    #[test]
    fn verification_test() {
        let mut deps = OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: CustomMockQuerier::default(),
            custom_query_type: PhantomData::<Empty>,
        };
        WORMHOLE_CORE
            .save(deps.as_mut().storage, &Addr::unchecked("wormhole_core"))
            .unwrap();

        let message = hex("03000000240001388100000000000000000000000004980c17e2ce26578c82f81207e706e4505fae3b0000a8690000000000000000000000000b1c1b54f45e02552331d3106e71f5e0b573d5d448656c6c6f21");
        let metadata = hex("010000000001004bda56f07ad553132013d612042cb51d38384bfd4ad8fe999d8290fc253c34487dd2f630c5d15628ab92ad1b1db50cb5033d81c0329a15a3f4d6d3ebc3dd111d01655dae28000000000002000000000000000000000000b94d10cdd2eb9fe9968cef953d5f6e1057e7c6400000000000000000c8488bdbb5bdbdcd0d6ff4fece2f5e5076538d786d9fa67ffb2ddb2e7f8450d990");
        let result = verify(deps.as_ref(), metadata, message).unwrap();
        assert_eq!(result, VerifyResponse { verified: true });
    }
}
