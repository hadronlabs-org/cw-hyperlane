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
// import { formatUnits, parseUnits } from '@ethersproject/units';
import { NodeHttpTransport } from '@improbable-eng/grpc-web-node-http-transport';
// import {
//   ASSOCIATED_TOKEN_PROGRAM_ID,
//   createAssociatedTokenAccountInstruction,
//   getAssociatedTokenAddress,
//   TOKEN_PROGRAM_ID,
// } from '@solana/spl-token';
// import {
//   Connection,
//   Keypair,
//   PublicKey,
//   TokenAccountsFilter,
//   Transaction,
// } from '@solana/web3.js';
// import { publicrpc } from '@certusone/wormhole-sdk-proto-web';
// import { BigNumber, ContractReceipt } from 'ethers';
// const { GrpcWebImpl, PublicRPCServiceClientImpl } = publicrpc;

export const ETH_NODE_URL = 'ws://localhost:8545';
export const ETH_PRIVATE_KEY =
  '0x6cbed15c793ce57650b9877cf6fa156fbef513c4e6134f022a85b1ffdd59b2a1'; // account 1
export const ETH_CORE_BRIDGE_ADDRESS =
  '0xC89Ce4735882C9F0f0FE26686c53074E09B0D550';
export const ETH_NFT_BRIDGE_ADDRESS =
  '0x26b4afb60d6c903165150c6f0aa14f8016be4aec';

// const SERVER_URL = 'http://localhost:';
// const CCQ_SERVER_URL = SERVER_URL + '6069/v1';
// const QUERY_URL = CCQ_SERVER_URL + '/query';
// const HEALTH_URL = SERVER_URL + '6068/health';
// const PRIVATE_KEY =
//   'cfb12303a19cde580bb4dd771639b0d26bc68353645571a8cff516ab2ee113a0';
// const WETH_ADDRESS = '0xDDb64fE46a91D46ee29420539FC25FD07c5FEa3E';

import {
  // approveEth,
  attestFromEth,
  // CHAIN_ID_ETH,
  // CHAIN_ID_SOLANA,
  CHAINS,
  CONTRACTS,
  // createWrappedOnSolana,
  getEmitterAddressEth,
  getSignedVAAWithRetry,
  // getForeignAssetSolana,
  // getIsTransferCompletedSolana,
  parseSequenceFromLogEth,
  // postVaaSolana,
  // redeemOnSolana,
  // transferFromEth,
  // tryNativeToUint8Array,
} from '@certusone/wormhole-sdk';

const WORMHOLE_RPC_URLS = ['http://localhost:7071']; // TODO: what is that?

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

  it('kek', async () => {
    // create a signer for Eth
    const provider = new ethers.WebSocketProvider(ETH_NODE_URL);
    const signer = new ethers.Wallet(ETH_PRIVATE_KEY, provider);
    // attest the test token
    const receipt = await attestFromEth(
      CONTRACTS.DEVNET.ethereum.token_bridge,
      signer,
      TEST_ERC20,
    );
    // get the sequence from the logs (needed to fetch the vaa)
    const sequence = parseSequenceFromLogEth(
      receipt,
      CONTRACTS.DEVNET.ethereum.core,
    );
    const emitterAddress = getEmitterAddressEth(
      CONTRACTS.DEVNET.ethereum.token_bridge,
    );
    // poll until the guardian(s) witness and sign the vaa
    const { vaaBytes: signedVAA } = await getSignedVAAWithRetry(
      WORMHOLE_RPC_URLS,
      CHAINS['ethereum'],
      emitterAddress,
      sequence,
      {
        transport: NodeHttpTransport(),
      },
    );
    // create a keypair for Solana
    // const keypair = Keypair.fromSecretKey(SOLANA_PRIVATE_KEY);
    // const payerAddress = keypair.publicKey.toString();
    // post vaa to Solana
  }, 1000000);
});

// // copypaste

// export const CHAINS = {
//   unset: 0,
//   solana: 1,
//   ethereum: 2,
//   terra: 3,
//   bsc: 4,
//   polygon: 5,
//   avalanche: 6,
//   oasis: 7,
//   algorand: 8,
//   aurora: 9,
//   fantom: 10,
//   karura: 11,
//   acala: 12,
//   klaytn: 13,
//   celo: 14,
//   near: 15,
//   moonbeam: 16,
//   neon: 17,
//   terra2: 18,
//   injective: 19,
//   osmosis: 20,
//   sui: 21,
//   aptos: 22,
//   arbitrum: 23,
//   optimism: 24,
//   gnosis: 25,
//   pythnet: 26,
//   xpla: 28,
//   btc: 29,
//   base: 30,
//   sei: 32,
//   rootstock: 33,
//   scroll: 34,
//   wormchain: 3104,
//   cosmoshub: 4000,
//   evmos: 4001,
//   kujira: 4002,
//   neutron: 4003,
//   celestia: 4004,
//   sepolia: 10002,
// } as const;

// export async function getSignedVAA(
//   host: string,
//   emitterChain: ChainID,
//   emitterAddress: string,
//   sequence: string,
//   extraGrpcOpts = {},
// ) {
//   const rpc = new GrpcWebImpl(host, extraGrpcOpts);
//   const api = new PublicRPCServiceClientImpl(rpc);
//   return await api.GetSignedVAA({
//     messageId: {
//       emitterChain,
//       emitterAddress,
//       sequence,
//     },
//   });
// }

// export async function getSignedVAAWithRetry(
//   hosts: string[],
//   emitterChain: ChainID,
//   emitterAddress: string,
//   sequence: string,
//   extraGrpcOpts = {},
//   retryTimeout = 1000,
//   retryAttempts?: number,
// ) {
//   let currentWormholeRpcHost = -1;
//   const getNextRpcHost = () => ++currentWormholeRpcHost % hosts.length;
//   let result;
//   let attempts = 0;
//   while (!result) {
//     attempts++;
//     await new Promise((resolve) => setTimeout(resolve, retryTimeout));
//     try {
//       result = await getSignedVAA(
//         hosts[getNextRpcHost()],
//         emitterChain,
//         emitterAddress,
//         sequence,
//         extraGrpcOpts,
//       );
//     } catch (e) {
//       if (retryAttempts !== undefined && attempts > retryAttempts) {
//         throw e;
//       }
//     }
//   }
//   return result;
// }

// export function parseSequencesFromLogEth(
//     receipt: ContractReceipt,
//     bridgeAddress: string
// ): string[] {
//   // TODO: dangerous!(?)
//   const bridgeLogs = receipt.logs.filter((l) => {
//     return l.address === bridgeAddress;
//   });
//   return bridgeLogs.map((bridgeLog) => {
//     const {
//       args: { sequence },
//     } = Implementation__factory.createInterface().parseLog(bridgeLog);
//     return sequence.toString();
//   });
// }
