import { BigNumber, ethers, utils } from 'ethers';
import { utils as ethersUtils } from 'ethers/lib/ethers';

/**
 * JS Implementation of solidity/contracts/libs/Message.sol#formatMessage
 * @returns Hex string of the packed message
 */
export const formatMessage = (
  version: number | BigNumber,
  nonce: number | BigNumber,
  originDomain: Domain,
  senderAddr: Address,
  destinationDomain: Domain,
  recipientAddr: Address,
  body: HexString,
): HexString => {
  senderAddr = addressToBytes32(senderAddr);
  recipientAddr = addressToBytes32(recipientAddr);

  return ethers.utils.solidityPack(
    ['uint8', 'uint32', 'uint32', 'bytes32', 'uint32', 'bytes32', 'bytes'],
    [
      version,
      nonce,
      originDomain,
      senderAddr,
      destinationDomain,
      recipientAddr,
      body,
    ],
  );
};

// ===== COPYPASTE FROM HYPERLANE

/**
 * Get ID given message bytes
 * @param message Hex string of the packed message (see formatMessage)
 * @returns Hex string of message id
 */
export function messageId(message: HexString): HexString {
  return ethers.utils.solidityKeccak256(['bytes'], [message]);
}

/**
 * Parse a serialized Hyperlane message from raw bytes.
 *
 * @param message
 * @returns
 */
export function parseMessage(message: string): ParsedMessage {
  const VERSION_OFFSET = 0;
  const NONCE_OFFSET = 1;
  const ORIGIN_OFFSET = 5;
  const SENDER_OFFSET = 9;
  const DESTINATION_OFFSET = 41;
  const RECIPIENT_OFFSET = 45;
  const BODY_OFFSET = 77;

  const buf = Buffer.from(utils.arrayify(message));
  const version = buf.readUint8(VERSION_OFFSET);
  const nonce = buf.readUInt32BE(NONCE_OFFSET);
  const origin = buf.readUInt32BE(ORIGIN_OFFSET);
  const sender = utils.hexlify(buf.slice(SENDER_OFFSET, DESTINATION_OFFSET));
  const destination = buf.readUInt32BE(DESTINATION_OFFSET);
  const recipient = utils.hexlify(buf.slice(RECIPIENT_OFFSET, BODY_OFFSET));
  const body = utils.hexlify(buf.slice(BODY_OFFSET));
  return { version, nonce, origin, sender, destination, recipient, body };
}

export enum ProtocolType {
  Ethereum = 'ethereum',
  Sealevel = 'sealevel',
  Fuel = 'fuel',
  Cosmos = 'cosmos',
}
// A type that also allows for literal values of the enum
export type ProtocolTypeValue = `${ProtocolType}`;

export const ProtocolSmallestUnit = {
  [ProtocolType.Ethereum]: 'wei',
  [ProtocolType.Sealevel]: 'lamports',
  [ProtocolType.Cosmos]: 'uATOM',
};

/********* BASIC TYPES *********/
export type Domain = number;
export type Address = string;
export type AddressBytes32 = string;
export type ChainCaip2Id = `${string}:${string}`; // e.g. ethereum:1 or sealevel:1399811149
export type TokenCaip19Id = `${string}:${string}/${string}:${string}`; // e.g. ethereum:1/erc20:0x6b175474e89094c44da98b954eedeac495271d0f
export type HexString = string;

// copied from node_modules/@ethersproject/bytes/src.ts/index.ts
export type SignatureLike =
  | {
      r: string;
      s?: string;
      _vs?: string;
      recoveryParam?: number;
      v?: number;
    }
  | ethers.utils.BytesLike;

export type MerkleProof = {
  branch: ethers.utils.BytesLike[];
  leaf: ethers.utils.BytesLike;
  index: number;
};

/********* HYPERLANE CORE *********/
export type Checkpoint = {
  root: string;
  index: number; // safe because 2 ** 32 leaves < Number.MAX_VALUE
  mailbox_domain: Domain;
  merkle_tree_hook_address: Address;
};

/**
 * Shape of a checkpoint in S3 as published by the agent.
 */
export type S3CheckpointWithId = {
  value: {
    checkpoint: Checkpoint;
    message_id: HexString;
  };
  signature: SignatureLike;
};

export type S3Checkpoint = {
  value: Checkpoint;
  signature: SignatureLike;
};

export type CallData = {
  to: Address;
  data: string;
};

export enum MessageStatus {
  NONE = 0,
  PROCESSED,
}

export type ParsedMessage = {
  version: number;
  nonce: number;
  origin: number;
  sender: string;
  destination: number;
  recipient: string;
  body: string;
};

export type ParsedLegacyMultisigIsmMetadata = {
  checkpointRoot: string;
  checkpointIndex: number;
  originMailbox: string;
  proof: ethers.utils.BytesLike[];
  signatures: ethers.utils.BytesLike[];
  validators: ethers.utils.BytesLike[];
};

export enum InterchainSecurityModuleType {
  MULTISIG = 3,
}

// For EVM addresses only, kept for backwards compatibility and convenience
export function addressToBytes32(address: Address): string {
  return ethersUtils
    .hexZeroPad(ethersUtils.hexStripZeros(address), 32)
    .toLowerCase();
}
