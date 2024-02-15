import { writeFileSync } from "fs";

import { loadContext } from "../src/load_context";
import {
  Client,
  HookType,
  IsmType,
  config,
  getSigningClient,
} from "../src/config";

import { ContractFetcher, Contracts } from "./fetch";
import { Context } from "../src/types";

const name = (c: any) => c.contractName;
const addr = (ctx: Context, c: any) => ctx.contracts[name(c)].address!;

async function main() {
  if (require.main !== module) {
    console.log("deploy required as module. Don't run")
    return;
  }
  const client = await getSigningClient(config);

  let ctx = loadContext(config.network.id);

  const contracts = new ContractFetcher(ctx, client).getContracts();
  const {
    core: { mailbox },
    mocks,
  } = contracts;

  ctx = await deploy_core(ctx, client, contracts);
  ctx = await deploy_igp(ctx, client, contracts);
  ctx = await deploy_ism_hook(ctx, client, contracts);

  // init test mock msg receiver
  ctx.contracts[name(mocks.receiver)] = await mocks.receiver.instantiate({
    hrp: config.network.hrp,
  });

  // pre-setup
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

  writeFileSync("./save.json", JSON.stringify(ctx, null, 2));
}

const deploy_core = async (
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

  // init validator announce
  ctx.contracts[name(va)] = await va.instantiate({
    hrp: config.network.hrp,
    mailbox: addr(ctx, mailbox),
  });

  return ctx;
};

const deploy_igp = async (
  ctx: Context,
  client: Client,
  { igp }: Contracts
): Promise<Context> => {
  // init igp
  ctx.contracts[name(igp.core)] = await igp.core.instantiate({
    hrp: config.network.hrp,
    owner: client.signer,
    gas_token: config.deploy.igp.token || config.network.gas.denom,
    beneficiary: client.signer,
  });

  // init igp oracle
  ctx.contracts[name(igp.oracle)] = await igp.oracle.instantiate({
    owner: client.signer,
  });

  await client.wasm.execute(
    client.signer,
    addr(ctx, igp.oracle),
    {
      set_remote_gas_data_configs: {
        configs: Object.entries(config.deploy.igp.configs).map(
          ([domain, v]) => ({
            remote_domain: Number(domain),
            token_exchange_rate: v.exchange_rate.toString(),
            gas_price: v.gas_price.toString(),
          })
        ),
      },
    },
    "auto"
  );

  await client.wasm.execute(
    client.signer,
    addr(ctx, igp.core),
    {
      router: {
        set_routes: {
          set: Object.keys(config.deploy.igp.configs).map((domain) => ({
            domain: Number(domain),
            route: addr(ctx, igp.oracle),
          })),
        },
      },
    },
    "auto"
  );

  return ctx;
};

const deploy_ism_hook = async (
  ctx: Context,
  client: Client,
  contracts: Contracts
) => {
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

  return ctx;
};

export const deploy_ism = async (
  ctx: Context,
  client: Client,
  ism: IsmType,
  contracts: Contracts
): Promise<string> => {
  const { isms } = contracts;

  let ism_addr;
  switch (ism.type) {
    case "multisig":
      const multisig_ism_res = await isms.multisig.instantiate({
        owner: ism.owner === "<signer>" ? client.signer : ism.owner,
      });

      await client.wasm.execute(
        client.signer,
        multisig_ism_res.address!,
        {
          enroll_validators: {
            set: Object.entries(ism.validators).flatMap(([domain, validator]) =>
              validator.addrs.map((v) => ({
                domain: Number(domain),
                validator: v,
              }))
            ),
          },
        },
        "auto"
      );

      await client.wasm.execute(
        client.signer,
        multisig_ism_res.address!,
        {
          set_thresholds: {
            set: Object.entries(ism.validators).map(
              ([domain, { threshold }]) => ({
                domain: Number(domain),
                threshold,
              })
            ),
          },
        },
        "auto"
      );
      ism_addr = multisig_ism_res.address!;
      break;

    case "aggregate":
      const aggregate_ism_addrs = [];
      for (let sub_ism of ism.isms) {
        const addr = await deploy_ism(ctx, client, sub_ism, contracts)
        aggregate_ism_addrs.push(addr)
      }

      const aggregate_ism_res = await isms.aggregate.instantiate({
        owner: ism.owner === "<signer>" ? client.signer : ism.owner,
        isms: aggregate_ism_addrs,
        threshold: ism.threshold,
      });

      ism_addr = aggregate_ism_res.address!;
      break;

    case "routing":
      const routing_ism_res = await isms.routing.instantiate({
        owner: ism.owner === "<signer>" ? client.signer : ism.owner,
      });

      await client.wasm.execute(
        client.signer,
        routing_ism_res.address!,
        {
          router: {
            set_routes: {
              set: await Promise.all(
                Object.entries(ism.isms).map(async ([domain, v]) => {
                  const route = await deploy_ism(ctx, client, v, contracts);
                  return { domain, route };
                })
              ),
            },
          },
        },
        "auto"
      );

      ism_addr = routing_ism_res.address!;
      break;

    case "wormhole":
      const wormhole_ism_res = await isms.wormhole.instantiate({
        owner: ism.owner === "<signer>" ? client.signer : ism.owner,
        wormhole_core: ism.wormhole_core,
        vaa_emitter_chain: ism.emitter_chain,
        hyperlane_origin_domain: ism.origin_domain,
      });

      ism_addr = wormhole_ism_res.address!;
      break;

    case "axelar":
      const axelar_ism_res = await isms.axelar.instantiate({
        owner: ism.owner === "<signer>" ? client.signer : ism.owner,
        axelar_hook_sender: ism.axelar_hook_sender,
        origin_address: ism.origin_address,
        origin_chain: ism.origin_chain,
      });

      ism_addr = axelar_ism_res.address!;
      break;


    default:
      throw new Error("invalid ism type");
  }

  ctx.contracts[`hpl_ism_${ism.type}`].address = ism_addr;

  return ism_addr;
};

export const deploy_hook = async (
  ctx: Context,
  client: Client,
  hook: HookType,
  contracts: Contracts
): Promise<string> => {
  const {
    core: { mailbox },
    hooks,
    igp,
    mocks,
  } = contracts;

  let hook_addr;
  switch (hook.type) {
    case "aggregate":
      const aggregate_hook_addrs = [];
      for (let sub_hook of hook.hooks) {
        const addr = await deploy_hook(ctx, client, sub_hook, contracts)
        aggregate_hook_addrs.push(addr)
      }
      const aggregate_hook_res = await hooks.aggregate.instantiate({
        owner: hook.owner === "<signer>" ? client.signer : hook.owner,
        hooks: aggregate_hook_addrs,
      });

      hook_addr = aggregate_hook_res.address!;
      break;

    case "merkle":
      const merkle_hook_res = await hooks.merkle.instantiate({
        owner: hook.owner === "<signer>" ? client.signer : hook.owner,
        mailbox: addr(ctx, mailbox),
      });

      hook_addr = merkle_hook_res.address!;
      break;

    case "mock":
      const mock_hook_res = await mocks.hook.instantiate({});

      hook_addr = mock_hook_res.address!;
      break;

    case "pausable":
      const pausable_hook_res = await hooks.pausable.instantiate({
        owner: hook.owner === "<signer>" ? client.signer : hook.owner,
      });

      hook_addr = pausable_hook_res.address!;
      break;

    case "igp":
      return ctx.contracts[name(igp.core)].address!;

    case "routing":
      const routing_hook_res = await hooks.routing.instantiate({
        owner: hook.owner === "<signer>" ? client.signer : hook.owner,
      });

      await client.wasm.execute(
        client.signer,
        routing_hook_res.address!,
        {
          router: {
            set_routes: {
              set: await Promise.all(
                Object.entries(hook.hooks).map(async ([domain, v]) => {
                  const route = await deploy_hook(ctx, client, v, contracts);
                  return { domain, route };
                })
              ),
            },
          },
        },
        "auto"
      );
      hook_addr = routing_hook_res.address!;
      break;

    case "wormhole":
      const wormhole_hook_res = await hooks.wormhole.instantiate({
        owner: hook.owner === "<signer>" ? client.signer : hook.owner,
        wormhole_core: hook.wormhole_core,
        mailbox: addr(ctx, mailbox),
      });
      hook_addr = wormhole_hook_res.address!;
      break;

    case "axelar":
      const axelar_hook_res = await hooks.axelar.instantiate({
        owner: hook.owner === "<signer>" ? client.signer : hook.owner,
        destination_chain: hook.destination_chain,
        destination_contract: hook.destination_contract,
        destination_ism: hook.destination_ism,
        axelar_gateway_channel: hook.axelar_gateway_channel,
        gas_token: hook.gas_token,
        mailbox: addr(ctx, mailbox),
      });
      hook_addr = axelar_hook_res.address!;
      break;

    default:
      throw new Error("invalid hook type");
  }

  let hook_name = hook.type !== "mock"
    ? `hpl_hook_${hook.type}`
    : "hpl_test_mock_hook";

  ctx.contracts[hook_name].address = hook_addr;

  return hook_addr;
};

main().catch(console.error);
