import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../src/testSuite';
import fs from 'fs';
import Cosmopark from '@neutron-org/cosmopark';
// import { Client as NeutronClient } from '@neutron-org/client-ts';
import { ethers } from 'ethers';
import { NodeHttpTransport } from '@improbable-eng/grpc-web-node-http-transport';
import { Implementation__factory } from '@certusone/wormhole-sdk/lib/esm/ethers-contracts';

// import { Address, Domain, HexString, ParsedMessage } from '@hyperlane-xyz/sdk/dist/utils';
// import { publicrpc } from '@certusone/wormhole-sdk-proto-web';
// const { GrpcWebImpl, PublicRPCServiceClientImpl } = publicrpc;

const ETH_RPC_URL = 'http://localhost:8545';
const ETH_PRIVATE_KEY =
  '0x6cbed15c793ce57650b9877cf6fa156fbef513c4e6134f022a85b1ffdd59b2a1'; // account 1

import {
  CHAINS,
  CONTRACTS,
  getEmitterAddressEth,
  getSignedVAAWithRetry,
  // ParsedVaa,
  parseSequenceFromLogEth,
  // parseVaa,
} from '@certusone/wormhole-sdk';
import { formatMessage, messageId } from '../src/helpers/hyperlane_copypaste';

const WORMHOLE_RPC_URLS = ['http://localhost:7071'];
const ETH_CHAIN_ID = 2; // TODO: check
const EMITTER_ADDRESS =
  '000000000000000000000000ffcf8fdee72ac11b5c542428b35eef5769c409f0';
const HYPERLANE_MESSAGE_ORIGIN_DOMAIN = 1; // TODO
const HYPERLANE_MESSAGE_ORIGIN_SENDER = '1'; // TODO

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
    senderAddr: 'todo',
    destinationDomain: 2,
    recipientAddr: 'todo',
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
    context.park = await setupPark('simple', ['neutron', 'gaia'], true);

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
      fs.readFileSync('../artifacts/wormhole_ibc.wasm'),
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
          addresses: [{ bytes: 'WMw65cCXshPOPIGXnhuflXB0aqU=' }],
          expiration_time: 0,
        },
        chain_id: 4003,
        fee_denom: 'untrn',
      },
      'wormholeIbc',
      'auto',
      {
        admin: deployer, // want to be able to migrate contract for testing purposes (set low timeout values)
      },
    );
    wormholeIbcAddress = wormholeIbcInstantiateRes.contractAddress;
    expect(wormholeIbcAddress).toBeTruthy();
  }, 1000000);

  it('deploys the neutron ISM contract', async () => {
    const neutronWormholeIsmRes = await wasmClient.upload(
      deployer,
      fs.readFileSync('../artifacts/neutron_ism_wormhole.wasm'),
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
        emitter_chain: ETH_CHAIN_ID,
        emitter_address: new TextEncoder().encode(EMITTER_ADDRESS),
        origin_domain: HYPERLANE_MESSAGE_ORIGIN_DOMAIN,
        origin_sender: new TextEncoder().encode(
          HYPERLANE_MESSAGE_ORIGIN_SENDER,
        ),
      },
      'wormholeIbc',
      'auto',
      {
        admin: deployer, // want to be able to migrate contract for testing purposes (set low timeout values)
      },
    );
    console.log('hex encoded: ' + new TextEncoder().encode(EMITTER_ADDRESS));
    neutronWormholeIsmAddress =
      neutronWormholeIsmInstantiateRes.contractAddress;
    expect(neutronWormholeIsmAddress).toBeTruthy();
  }, 1000000);

  let signedVAA: Uint8Array;
  // let parsedVaa: ParsedVaa;

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

    // parsedVaa = parseVaa(signedVAA);
  }, 1000000);

  it('submits the VAA message with hyperlane message to verify to Neutron Wormhole ISM contract', async () => {
    const res = await wasmClient.execute(
      deployer,
      neutronWormholeIsmAddress,
      {
        submit_meta: {
          metadata: Buffer.from(signedVAA as Uint8Array).toString('hex'),
          message: hexHyperlaneMessage,
        },
      },
      'auto',
      '',
      [{ amount: '8000', denom: 'untrn' }],
    );
    expect(res.events.length).toBeGreaterThan(0);
    console.log(
      'submit_meta result: \n' + JSON.stringify(res.logs) + '\n\n\n\n',
    );
  }, 1000000);

  it('verifies submitted message successfully', async () => {
    const res = await wasmClient.queryContractSmart(neutronWormholeIsmAddress, {
      verify: {
        metadata: Buffer.from(signedVAA as Uint8Array).toString('hex'),
        message: hexHyperlaneMessage,
      },
    });

    expect(res.verified).toBeTruthy();
  });
});
