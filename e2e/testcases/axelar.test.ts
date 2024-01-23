import { describe, expect, it, beforeAll, afterAll } from 'vitest';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { DirectSecp256k1HdWallet } from '@cosmjs/proto-signing';
import { GasPrice } from '@cosmjs/stargate';
import { setupPark } from '../src/testSuite';
import Cosmopark from '@neutron-org/cosmopark';
import {
  // createAndExport,
  // createNetwork,
  EvmRelayer,
  Network,
  relay,
  RelayerType,
  setupNetwork,
} from '@axelar-network/axelar-local-dev';
import {
  // defaultAxelarChainInfo,
  AxelarRelayerService,
} from '@axelar-network/axelar-local-dev-cosmos/';
import { ethers /*, Wallet*/ } from 'ethers';

const TILTNET_ETH_RPC_URL = 'http://localhost:8545';
const TILTNET_ETH_PRIVATE_KEY =
  '0x6cbed15c793ce57650b9877cf6fa156fbef513c4e6134f022a85b1ffdd59b2a1'; // account 1

describe('Test Wormhole ISM', () => {
  const context: { park?: Cosmopark } = {};

  let wasmClient: SigningCosmWasmClient;
  // let neutronClient: any;
  let deployer: string;

  let wormholeIbcAddress: string;

  const dropConnections = [];

  beforeAll(async () => {
    // start neutron, gaia and hermes relayer
    context.park = await setupPark('simple', ['axelar', 'neutron'], false);

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

  it('starts the axelar EVM relayer', async () => {
    const evmRelayer = new EvmRelayer();
    const axelarChainInfo = {
      prefix: 'axelar',
      denom: 'uaxl',
      lcdUrl: `http://localhost:${context.park.ports['axelar'].rest}`,
      rpcUrl: `http://localhost:${context.park.ports['axelar'].rpc}`,
      wsUrl: 'ws://localhost:/axelar-rpc/websocket',
    };
    // export interface NetworkOptions {
    //   ganacheOptions?: any;
    //   dbPath?: string;
    //   port?: number;
    //   name?: string;
    //   chainId?: number;
    //   seed?: string;
    // }
    // const evmNetwork = await createNetwork({
    //   port: 8545,
    //   name: 'Ethereum',
    //   seed: '',
    // });
    const provider = new ethers.providers.WebSocketProvider(
      TILTNET_ETH_RPC_URL,
    );
    const signer = new ethers.Wallet(TILTNET_ETH_PRIVATE_KEY, provider);
    const network = await setupNetwork(`http://localhost:8545`, {
      ownerKey: signer,
    });

    validateNetwork(network);
    const wasmRelayer = await AxelarRelayerService.create(axelarChainInfo);

    evmRelayer.setRelayer(RelayerType.Wasm, wasmRelayer);

    const ibcRelayer = wasmRelayer.ibcRelayer;

    // const cosmosConfig = {
    //   srcChannelId: ibcRelayer.srcChannelId,
    //   dstChannelId: ibcRelayer.destChannelId,
    // };

    // // Setup for Ethereum Network and Wasm chain relayer
    // const ethereumNetwork = await createEthereumNetwork({ name: "Ethereum" });
    //
    // // Deploy Smart Contract on the EVM (Ethereum Virtual Machine)
    // const ethereumContract = await deployEthereumContract(
    //     ethereumNetwork.userWallets[0],
    //     SendReceiveContract,
    //     [
    //       ethereumNetwork.gateway.address,
    //       ethereumNetwork.gasService.address,
    //       "Ethereum",
    //     ]
    // );

    // Relay messages between Ethereum and Wasm chains
    await relay({
      wasm: wasmRelayer,
      evm: evmRelayer,
    });

    dropConnections.push(() => ibcRelayer.stopInterval());
    dropConnections.push(() => wasmRelayer.stopListening());
  }, 1000000);

  afterAll(async () => {
    if (context.park) {
      await context.park.stop();
    }
    for (const dropConnection of dropConnections) {
      await dropConnection();
    }
  });
});

function validateNetwork(network: Network) {
  // wallets
  expect(network.provider).to.not.be.undefined;
  expect(network.userWallets).to.not.be.undefined;
  expect(network.ownerWallet).to.not.be.undefined;
  expect(network.operatorWallet).to.not.be.undefined;
  expect(network.adminWallets).to.not.be.undefined;

  // contracts
  expect(network.gasService).to.not.be.undefined;
  expect(network.constAddressDeployer).to.not.be.undefined;
  expect(network.create3Deployer).to.not.be.undefined;
  expect(network.gateway).to.not.be.undefined;
  expect(network.its).to.not.be.undefined;
  expect(network.interchainTokenService).to.not.be.undefined;
}
