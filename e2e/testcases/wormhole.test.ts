import { describe, expect, it, beforeAll, afterAll } from 'vitest';
// import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
// import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
// import { GasPrice } from '@cosmjs/stargate';
// import { setupPark } from '../src/testSuite';
// import fs from 'fs';
// import Cosmopark from '@neutron-org/cosmopark';
// import { Client as NeutronClient } from '@neutron-org/client-ts';
// import { V1IdentifiedChannel } from '@neutron-org/client-ts/src/ibc.core.channel.v1/rest';
// import { getIBCDenom } from '../src/helpers/ibc_denom';
// import { waitFor } from '../src/helpers/sleep';
// import { GaiaClient } from '../src/helpers/gaia_client';
import { ethers } from 'ethers';
import { NodeHttpTransport } from '@improbable-eng/grpc-web-node-http-transport';
import { Implementation__factory } from '@certusone/wormhole-sdk/lib/esm/ethers-contracts';
// import { publicrpc } from '@certusone/wormhole-sdk-proto-web';
// const { GrpcWebImpl, PublicRPCServiceClientImpl } = publicrpc;

const ETH_RPC_URL = 'http://localhost:8545';
export const ETH_PRIVATE_KEY =
  '0x6cbed15c793ce57650b9877cf6fa156fbef513c4e6134f022a85b1ffdd59b2a1'; // account 1
export const ETH_CORE_BRIDGE_ADDRESS =
  '0xC89Ce4735882C9F0f0FE26686c53074E09B0D550';

// const SERVER_URL = 'http://localhost:';
// const CCQ_SERVER_URL = SERVER_URL + '6069/v1';
// const QUERY_URL = CCQ_SERVER_URL + '/query';
// const HEALTH_URL = SERVER_URL + '6068/health';
// const PRIVATE_KEY =
//   'cfb12303a19cde580bb4dd771639b0d26bc68353645571a8cff516ab2ee113a0';
// const WETH_ADDRESS = '0xDDb64fE46a91D46ee29420539FC25FD07c5FEa3E';

import {
  // approveEth,
  // CHAIN_ID_ETH,
  // CHAIN_ID_SOLANA,
  CHAINS,
  CONTRACTS,
  // createWrappedOnSolana,
  getEmitterAddressEth,
  getSignedVAAWithRetry,
  // getForeignAssetSolana,
  // getIsTransferCompletedSolana,
  parseSequenceFromLogEth, parseVaa,
  // postVaaSolana,
  // redeemOnSolana,
  // transferFromEth,
  // tryNativeToUint8Array,
} from '@certusone/wormhole-sdk';

const WORMHOLE_RPC_URLS = ['http://localhost:7071'];

describe('Test Wormhole ISM', () => {
  // const context: { park?: Cosmopark } = {};
  //
  // let client: SigningCosmWasmClient;
  // let neutronClient: any;
  // let hubClient: any;
  // let deployer: string;
  //
  // let claimerCodeId: number;
  // let claimerAddress: string;
  //
  // let transferChannel: V1IdentifiedChannel;
  //
  // let ibcDenom: string;

  beforeAll(async () => {
    // start neutron, gaia and hermes relayer
    // context.park = await setupPark('simple', ['neutron', 'gaia'], true)
    //
    // const mnemonic = context.park.config.wallets.demowallet1.mnemonic
    // const endpoint = `http://127.0.0.1:${context.park.ports['neutron'].rpc}`
    // const options = {gasPrice: GasPrice.fromString('0.025untrn')}
    // const walletOptions = {prefix: 'neutron'}
    // const wallet = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, walletOptions)
    // client = await SigningCosmWasmClient.connectWithSigner(endpoint, wallet, options)
    //
    // // deployer will deploy and manage all of our contracts for simplicity
    // const accounts = await wallet.getAccounts()
    // deployer = accounts[0].address
    //
    // neutronClient = new NeutronClient({
    //     apiURL: `http://127.0.0.1:${context.park.ports['neutron'].rest}`,
    //     rpcURL: `127.0.0.1:${context.park.ports['neutron'].rpc}`,
    //     prefix: 'neutron',
    // })
    // hubClient = new GaiaClient({
    //     apiURL: `http://127.0.0.1:${context.park.ports['gaia'].rest}`,
    //     rpcURL: `127.0.0.1:${context.park.ports['gaia'].rpc}`,
    //     prefix: 'cosmos'
    // })
    // const channelsRes = await neutronClient.IbcCoreChannelV1.query.queryChannels()
    // transferChannel = channelsRes.data.channels.find(c => c.port_id === 'transfer' && c.state === 'STATE_OPEN')
    // expect(transferChannel).toBeDefined()
    // expect(transferChannel.port_id).toEqual('transfer')
  }, 1000000);

  afterAll(async () => {
    // if (context.park) {
    //     await context.park.stop()
    // }
  });

  it('deploys the wormhole core contracts', async () => {
    // let connectionId = transferChannel.connection_hops[0]
    //
    // const claimerStoreRes = await client.upload(
    //     deployer,
    //     fs.readFileSync('../artifacts/neutron_airdrop_transfer.wasm'),
    //     1.5,
    // )
    // claimerCodeId = claimerStoreRes.codeId
    // expect(claimerCodeId).toBeGreaterThan(0)
    //
    // const claimerRes = await client.instantiate(deployer, claimerCodeId, {
    //     connection_id: connectionId,
    //     transfer_channel_id: transferChannel.channel_id, // neutron to cosmoshub transfer channel id
    //     ibc_neutron_denom: ibcDenom,
    //     ibc_timeout_seconds: 3600 * 5,
    //     amount: '14000',
    // }, 'credits', 'auto', {
    //     admin: deployer // want to be able to migrate contract for testing purposes (set low timeout values)
    // })
    // claimerAddress = claimerRes.contractAddress
    // expect(claimerAddress).toBeTruthy()
  }, 1000000);

  it('publishes the message', async () => {
    // create a signer for Eth
    const provider = new ethers.providers.WebSocketProvider(ETH_RPC_URL);
    const signer = new ethers.Wallet(ETH_PRIVATE_KEY, provider);

    console.log('signer created');
    const wormhole = Implementation__factory.connect(
      CONTRACTS.DEVNET.ethereum.core,
      signer,
    );
    console.log('connected to wormhole');
    const payload = '0x123123';
    const msgTx = await wormhole.publishMessage(0, payload, 1);
    console.log('published msgTx');
    console.log('msgTx: \n' + msgTx.raw + '\n\n\n');
    const receipt = await msgTx.wait();
    console.log('waited for receipt: \n' + JSON.stringify(receipt.logs));
    console.log('events: ' + JSON.stringify(receipt.events));

    const sequence = parseSequenceFromLogEth(
      receipt,
      CONTRACTS.DEVNET.ethereum.core,
    );
    console.log('sequence: ' + sequence);
    // const emitterAddress = getEmitterAddressEth(/*CONTRACTS.DEVNET.ethereum.core*/'0xFFcf8FDEE72ac11b5c542428B35EEF5769C409f0');
    const emitterAddress = getEmitterAddressEth(signer.address);
    console.log('emitter: ' + emitterAddress);

    // const rpc = new GrpcWebImpl(ETH_RPC_URL, {});
    // const api = new PublicRPCServiceClientImpl(rpc);
    // const heart = await api.GetSignedVAA()

    // poll until the guardian(s) witness and sign the vaa
    const chain = CHAINS['ethereum'];

    const { vaaBytes: signedVAA } = await getSignedVAAWithRetry(
      WORMHOLE_RPC_URLS,
      chain,
      emitterAddress,
      sequence,
      {
        transport: NodeHttpTransport(),
      },
    );
    expect(signedVAA).not.toBeNull();
    const parsed = parseVaa(signedVAA);
    console.log('parsedVaa: ' + JSON.stringify(parsed.emitterChain));

    // create a keypair for Solana
    // const keypair = Keypair.fromSecretKey(SOLANA_PRIVATE_KEY);
    // const payerAddress = keypair.publicKey.toString();
    // post vaa to Solana
  }, 1000000);
});

// const getArbitraryBytes32 = (): string =>
//   ethers.hexlify(ethers.toUtf8Bytes('asdfasdfasdf'));
