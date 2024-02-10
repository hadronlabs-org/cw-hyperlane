import { loadContext } from "../src/load_context";
import { config, getSigningClient } from "../src/config";

import HplWarpNative from "../src/contracts/hpl_warp_native";
import HplIsmMultisig from "../src/contracts/hpl_ism_multisig";
import { ContractFetcher } from "./fetch";
import { addPad } from "../src/conv";

async function main() {
  const client = await getSigningClient(config);

  const ctx = loadContext(config.network.id);

  const fetcher = new ContractFetcher(ctx, client);

  // const ism_multisig = fetcher.get(HplIsmMultisig, "hpl_ism_multisig");

  let res;

  // res = await ism_multisig.execute({
  //   enroll_validator: {
  //     set: {
  //       domain: 5,
  //       validator: client.signer,
  //       validator_pubkey: client.signer_pubkey,
  //     },
  //   },
  // });
  // console.log(res.events.filter((v) => v.type.startsWith("wasm")));

  const warp_native_ibc = fetcher.get(HplWarpNative, "hpl_warp_native");

  res = await warp_native_ibc.execute({
    router: {
      set_route: {
        set: {
          //TODO: temp for sepolia deploy
          domain: 11155111,
          route: addPad("0x8014Ea96F219A59C183cec9794039Bbe3167A847"),
        },
      },
    },
  });
  console.log(res.events.filter((v) => v.type.startsWith("wasm")));
}

main().catch(console.error);
