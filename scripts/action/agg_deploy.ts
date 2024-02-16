import { writeFileSync } from "fs";

import { ContractFetcher, Contracts } from "./fetch";
import { loadContext } from "../src/load_context";
import {
  Client,
  HookType,
  IsmType,
  config,
  getSigningClient,
} from "../src/config";
import { Context } from "../src/types";
import { addPad } from "../src/conv";
import { deploy_hook, deploy_ism } from "./deploy";
// import {parseWasmEventLog} from "./warp";
import { Contract } from "web3";

const name = (c: any) => c.contractName;
const addr = (ctx: Context, c: any) => ctx.contracts[name(c)].address!;
const WARP_DENOM = "weth"

async function main() {
  const client = await getSigningClient(config);

  let ctx = loadContext(config.network.id);

  const contracts = new ContractFetcher(ctx, client).getContracts();
  const {
    core: { mailbox },
    mocks,
  } = contracts;

  // ctx = await deploy_mailbox(ctx, client, contracts);
  // ctx = await deploy_va(ctx, client, contracts);
  ctx = await deploy_ism_and_hook(ctx, client, contracts);
  ctx = await deploy_warp(ctx, client, contracts);

  writeFileSync("./save.json", JSON.stringify(ctx, null, 2));

}

const deploy_mailbox = async (
  ctx: Context,
  client: Client,
  { core: { mailbox, va } }: Contracts
): Promise<Context> => {
  // init mailbox
  ctx.contracts[name(mailbox)] = await mailbox.instantiate({
    hrp: config.network.hrp,
    owner: client.signer,
    domain: config.network.domain,
  });

  return ctx;
};

const deploy_ism_and_hook = async (
  ctx: Context,
  client: Client,
  contracts: Contracts
) => {
  const mailbox = contracts.core.mailbox
  ctx.contracts["hpl_default_ism"] = {
    ...ctx.contracts[`hpl_ism_${config.deploy.ism?.type || "multisig"}`],

    address: await deploy_ism(
      ctx,
      client,
      config.deploy.ism || {
        type: "multisig",
        owner: "<signer>",
        validators: {
          5: {
            addrs: [client.signer_addr],
            threshold: 1,
          },
        },
      },
      contracts
    ),
  };

  ctx.contracts["hpl_default_hook"] = {
    ...ctx.contracts[
    config.deploy.hooks?.default?.type &&
      config.deploy.hooks?.default?.type !== "mock"
      ? `hpl_hook_${config.deploy.hooks.default.type}`
      : "hpl_test_mock_hook"
    ],

    address: await deploy_hook(
      ctx,
      client,
      config.deploy.hooks?.default || { type: "mock" },
      contracts
    ),
  };

  ctx.contracts["hpl_required_hook"] = {
    ...ctx.contracts[
    config.deploy.hooks?.required?.type &&
      config.deploy.hooks?.required?.type !== "mock"
      ? `hpl_hook_${config.deploy.hooks.required.type}`
      : "hpl_test_mock_hook"
    ],

    address: await deploy_hook(
      ctx,
      client,
      config.deploy.hooks?.required || { type: "mock" },
      contracts
    ),
  };

  await client.wasm.executeMultiple(
    client.signer,
    [
      {
        contractAddress: addr(ctx, mailbox),
        msg: {
          set_default_ism: {
            ism: ctx.contracts["hpl_default_ism"].address!,
          },
        },
      },
      {
        contractAddress: addr(ctx, mailbox),
        msg: {
          set_default_hook: {
            hook: ctx.contracts["hpl_default_hook"].address!,
          },
        },
      },
      {
        contractAddress: addr(ctx, mailbox),
        msg: {
          set_required_hook: {
            hook: ctx.contracts["hpl_required_hook"].address!,
          },
        },
      },
    ],
    "auto"
  );

  return ctx;
};

const deploy_va = async (
  ctx: Context,
  client: Client,
  { core: { mailbox, va } }: Contracts
): Promise<Context> => {

  // init validator announce
  ctx.contracts[name(va)] = await va.instantiate({
    hrp: config.network.hrp,
    mailbox: addr(ctx, mailbox),
  });

  return ctx;
};

const deploy_warp = async (
  ctx: Context,
  client: Client,
  contracts: Contracts
): Promise<Context> => {
  const { warp, core: { mailbox } } = contracts
  ctx.contracts[name(warp.native)] = await warp.native.instantiate({
    token: {
      bridged: {
        denom: WARP_DENOM,
      },
    },
    hrp: config.network.hrp,
    owner: client.signer,
    mailbox: mailbox.address!
  });
  const hook_addr = await deploy_hook(ctx, client, config.deploy.warp_route?.hook!, contracts)
  const ism_addr = await deploy_ism(ctx, client, config.deploy.warp_route?.ism!, contracts)
  await link_warp(ctx, client, contracts)

  let resp = await client.wasm.execute(
    client.signer,
    warp.native.address!,
    {
      connection: {
        set_ism: {
          ism: ism_addr
        },
      },
    },
    "auto"
  );
  //   console.log(parseWasmEventLog(resp));
  console.log(resp.transactionHash);

  resp = await client.wasm.execute(
    client.signer,
    warp.native.address!,
    {
      connection: {
        set_hook: {
          hook: hook_addr
        },
      },
    },
    "auto"
  );
  //   console.log(parseWasmEventLog(resp));
  console.log(resp.transactionHash);

  return ctx

};

async function link_warp(
  ctx: Context,
  client: Client,
  { warp }: Contracts) {

  const resp = await client.wasm.execute(
    client.signer,
    warp.native.address!,
    {
      router: {
        set_route: {
          set: {
            domain: config.deploy.warp_route?.domain,
            route: addPad(config.deploy.warp_route?.external_route!),
          },
        },
      },
    },
    "auto"
  );
  // console.log(parseWasmEventLog(resp));
  console.log(resp.transactionHash);
}

main().catch(console.error);
