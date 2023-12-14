import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../src/testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
import { ethers } from 'ethers';
import { NodeHttpTransport } from '@improbable-eng/grpc-web-node-http-transport';
import { Implementation__factory } from '@certusone/wormhole-sdk/lib/esm/ethers-contracts';
// import { Client as NeutronClient } from '@neutron-org/client-ts';

const ETH_RPC_URL = 'http://localhost:8545';
const ETH_PRIVATE_KEY =
  '0x6cbed15c793ce57650b9877cf6fa156fbef513c4e6134f022a85b1ffdd59b2a1'; // account 1

// consts.ts
export const TILTNET_GUARDIAN_PUBKEY = 'vvpCnVfNGLf4pNkaLamrSvBdD74=';

import {
  CHAINS,
  CONTRACTS,
  getEmitterAddressEth,
  getSignedVAAWithRetry,
  parseSequenceFromLogEth,
  parseVaa,
} from '@certusone/wormhole-sdk';
import { formatMessage, messageId } from '../src/helpers/hyperlane_copypaste';

// tilt up devnet `WHAT?` address
const WORMHOLE_RPC_URLS = ['http://localhost:7071'];
const EMITTER_ADDRESS_PADDED =
  '000000000000000000000000ffcf8fdee72ac11b5c542428b35eef5769c409f0';
const EMITTER_ADDRESS = '0xFFcf8FDEE72ac11b5c542428B35EEF5769C409f0';
const HYPERLANE_MESSAGE_ORIGIN_DOMAIN = 1; // Q: what will it be?
const HYPERLANE_MESSAGE_ORIGIN_SENDER = EMITTER_ADDRESS; // Q: will hyperlane sender === wormhole emitter?
const HYPERLANE_MESSAGE_ORIGIN_SENDER_PADDED = EMITTER_ADDRESS_PADDED;
const HYPERLANE_MESSAGE_ORIGIN_RECIPIENT =
  '0xFFcf8FDEE72ac11b5c542428B35EEF5769C409f0'; // Q: what will it be?
const WORMHOLE_NEUTRON_CHAIN_ID = 4003;

describe('Test Wormhole ISM', () => {
  const context: { park?: Cosmopark } = {};

  let wasmClient: SigningCosmWasmClient;
  // let neutronClient: any;
  let deployer: string;

  let wormholeIbcAddress: string;
  let neutronWormholeIsmAddress: string;

  // TODO: use real message
  const hyperlaneMessage = {
    version: 1,
    nonce: 1,
    originDomain: 1,
    senderAddr: HYPERLANE_MESSAGE_ORIGIN_SENDER.toLowerCase(),
    destinationDomain: 2,
    recipientAddr: HYPERLANE_MESSAGE_ORIGIN_RECIPIENT.toLowerCase(),
    body: '0x123123',
  };
  const hexHyperlaneMessage = formatMessage(
    hyperlaneMessage.version,
    hyperlaneMessage.nonce,
    hyperlaneMessage.originDomain,
    hyperlaneMessage.senderAddr,
    hyperlaneMessage.destinationDomain,
    hyperlaneMessage.recipientAddr,
    hyperlaneMessage.body,
  );
  const hyperlaneMessageId = messageId(hexHyperlaneMessage);

  beforeAll(async () => {
    // start neutron, gaia and hermes relayer
    context.park = await setupPark('simple', ['neutron'], false);

    const mnemonic = context.park.config.wallets.demowallet1.mnemonic;
    const endpoint = `http://127.0.0.1:${context.park.ports['neutron'].rpc}`;
    const options = { gasPrice: GasPrice.fromString('0.025untrn') };
    const walletOptions = { prefix: 'neutron' };
    const wallet = await DirectSecp256k1HdWallet.fromMnemonic(
      mnemonic,
      walletOptions,
    );
    wasmClient = await SigningCosmWasmClient.connectWithSigner(
      endpoint,
      wallet,
      options,
    );

    // deployer will deploy and manage all of our contracts for simplicity
    const accounts = await wallet.getAccounts();
    deployer = accounts[0].address;

    // neutronClient = new NeutronClient({
    //   apiURL: `http://127.0.0.1:${context.park.ports['neutron'].rest}`,
    //   rpcURL: `127.0.0.1:${context.park.ports['neutron'].rpc}`,
    //   prefix: 'neutron',
    // });
  }, 1000000);

  afterAll(async () => {
    if (context.park) {
      await context.park.stop();
    }
  });

  it('deploys the wormhole core contracts', async () => {
    const wormholeIbcRes = await wasmClient.upload(
      deployer,
      fs.readFileSync('./artifacts/contracts/wormhole_ibc-aarch64.wasm'),
      1.5,
    );
    const wormholeIbcCodeId = wormholeIbcRes.codeId;
    expect(wormholeIbcCodeId).toBeGreaterThan(0);

    const wormholeIbcInstantiateRes = await wasmClient.instantiate(
      deployer,
      wormholeIbcCodeId,
      {
        gov_chain: 1,
        gov_address: 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQ=',
        guardian_set_expirity: 86400,
        initial_guardian_set: {
          addresses: [{ bytes: TILTNET_GUARDIAN_PUBKEY }],
          expiration_time: 0,
        },
        chain_id: WORMHOLE_NEUTRON_CHAIN_ID,
        fee_denom: 'untrn',
      },
      'wormholeIbc',
      'auto',
    );
    wormholeIbcAddress = wormholeIbcInstantiateRes.contractAddress;
    expect(wormholeIbcAddress).toBeTruthy();
  }, 1000000);

  it('deploys the neutron ISM contract', async () => {
    const neutronWormholeIsmRes = await wasmClient.upload(
      deployer,
      fs.readFileSync('../artifacts/hpl_ism_wormhole-aarch64.wasm'),
      1.5,
    );
    const neutronIsmCodeId = neutronWormholeIsmRes.codeId;
    expect(neutronIsmCodeId).toBeGreaterThan(0);

    const neutronWormholeIsmInstantiateRes = await wasmClient.instantiate(
      deployer,
      neutronIsmCodeId,
      {
        owner: deployer,
        wormhole_core: wormholeIbcAddress,
        vaa_emitter_chain: CHAINS['ethereum'],
        vaa_emitter_address: EMITTER_ADDRESS_PADDED,
        hyperlane_origin_domain: HYPERLANE_MESSAGE_ORIGIN_DOMAIN,
        hyperlane_origin_sender:
          HYPERLANE_MESSAGE_ORIGIN_SENDER_PADDED.toLowerCase(),
      },
      'wormholeIbc',
      'auto',
    );
    neutronWormholeIsmAddress =
      neutronWormholeIsmInstantiateRes.contractAddress;
    expect(neutronWormholeIsmAddress).toBeTruthy();
  }, 1000000);

  let signedVAA: Uint8Array;

  it('publishes the VAA wormhole message', async () => {
    // create a signer for Eth
    const provider = new ethers.providers.WebSocketProvider(ETH_RPC_URL);
    const signer = new ethers.Wallet(ETH_PRIVATE_KEY, provider);

    const wormhole = Implementation__factory.connect(
      CONTRACTS.DEVNET.ethereum.core,
      signer,
    );
    // `hyperlaneMessageId` is the message payload expected
    const msgTx = await wormhole.publishMessage(0, hyperlaneMessageId, 1);
    const receipt = await msgTx.wait();

    // poll until the guardian(s) witness and sign the vaa
    const { vaaBytes } = await getSignedVAAWithRetry(
      WORMHOLE_RPC_URLS,
      CHAINS['ethereum'],
      getEmitterAddressEth(signer.address),
      parseSequenceFromLogEth(receipt, CONTRACTS.DEVNET.ethereum.core),
      { transport: NodeHttpTransport() },
    );
    signedVAA = vaaBytes;
    expect(signedVAA).not.toBeNull();

    const parsedVaa = parseVaa(signedVAA);
    expect(parsedVaa.payload.toString('hex')).toEqual(
      hyperlaneMessageId.slice(2),
    );
  }, 1000000);

  it('verifies vaa through wormhole core', async () => {
    const res = await wasmClient.queryContractSmart(wormholeIbcAddress, {
      verify_v_a_a: {
        vaa: Buffer.from(signedVAA as Uint8Array).toString('base64'),
        block_time: 0,
      },
    });
    // console.log('query res: ' + JSON.stringify(res));
    // TODO: validate it
    expect(res.emitter_chain).toEqual(CHAINS['ethereum']);
    // expect(res.sequence).toEqual();
    // expect(res.emitter_address).toEqual();
  });

  it('submits the VAA message with hyperlane message to verify to Neutron Wormhole ISM contract', async () => {
    const res = await wasmClient.execute(
      deployer,
      neutronWormholeIsmAddress,
      {
        submit_meta: {
          metadata: Buffer.from(signedVAA as Uint8Array).toString('hex'),
          message: hexHyperlaneMessage.slice(2),
        },
      },
      'auto',
    );
    expect(res.events.length).toBeGreaterThan(0);
    // extract packed_id from events
    const packed_id = res.logs[0].events
      .flatMap((e) => e.attributes)
      .find((a) => a.key === 'packed_id').value;
    expect(packed_id).toEqual(hyperlaneMessageId.slice(2));
  }, 1000000);

  it('verifies submitted message successfully', async () => {
    const res = await wasmClient.queryContractSmart(neutronWormholeIsmAddress, {
      ism: {
        verify: {
          metadata: Buffer.from(signedVAA as Uint8Array).toString('hex'),
          message: hexHyperlaneMessage.slice(2),
        },
      },
    });

    expect(res.verified).toBeTruthy();
  });
});
