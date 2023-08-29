import { KernelCircuitPublicInputs, MAX_NEW_CONTRACTS_PER_TX, Proof, PublicCallRequest } from '@aztec/circuits.js';
import { Tuple, numToUInt32BE } from '@aztec/foundation/serialize';
import { ExtendedContractData, Tx, TxHash, TxL2Logs } from '@aztec/types';

/**
 * Enumeration of P2P message types.
 */
export enum Messages {
  POOLED_TRANSACTIONS = 1,
  POOLED_TRANSACTION_HASHES = 2,
  GET_TRANSACTIONS = 3,
}

/**
 * Create a P2P message from the message type and message data.
 * @param type - The type of the message.
 * @param messageData - The binary message data.
 * @returns The encoded message.
 */
export function createMessage(type: Messages, messageData: Buffer) {
  return Buffer.concat([numToUInt32BE(type), messageData]);
}

/**
 * Create a POOLED_TRANSACTIONS message from an array of transactions.
 * @param txs - The transactions to encoded into a message.
 * @returns The encoded message.
 */
export function createTransactionsMessage(txs: Tx[]) {
  const messageData = txs.map(toTxMessage);
  return createMessage(Messages.POOLED_TRANSACTIONS, Buffer.concat(messageData));
}

/**
 * Decode a POOLED_TRANSACTIONS message into the original transaction objects.
 * @param message - The binary message to be decoded.
 * @returns - The array of transactions originally encoded into the message.
 */
export function decodeTransactionsMessage(message: Buffer) {
  const lengthSize = 4;
  let offset = 0;
  const txs: Tx[] = [];
  while (offset < message.length) {
    const dataSize = message.readUInt32BE(offset);
    const totalSizeOfMessage = lengthSize + dataSize;
    txs.push(fromTxMessage(message.subarray(offset, offset + totalSizeOfMessage)));
    offset += totalSizeOfMessage;
  }
  return txs;
}

/**
 * Create a POOLED_TRANSACTION_HASHES message.
 * @param hashes - The transaction hashes to be sent.
 * @returns The encoded message.
 */
export function createTransactionHashesMessage(hashes: TxHash[]) {
  const messageData = hashes.map(x => x.buffer);
  return createMessage(Messages.POOLED_TRANSACTION_HASHES, Buffer.concat(messageData));
}

/**
 * Decode a POOLED_TRANSACTION_HASHESs message ito the original transaction hash objects.
 * @param message - The binary message to be decoded.
 * @returns - The array of transaction hashes originally encoded into the message.
 */
export function decodeTransactionHashesMessage(message: Buffer) {
  let offset = 0;
  const txHashes: TxHash[] = [];
  while (offset < message.length) {
    const slice = message.subarray(offset, offset + TxHash.SIZE);
    if (slice.length < TxHash.SIZE) {
      throw new Error(`Invalid message size when processing transaction hashes message`);
    }
    txHashes.push(new TxHash(slice));
    offset += TxHash.SIZE;
  }
  return txHashes;
}

/**
 * Create a GET_TRANSACTIONS message from an array of transaction hashes.
 * @param hashes - The hashes of the transactions to be requested.
 * @returns The encoded message.
 */
export function createGetTransactionsRequestMessage(hashes: TxHash[]) {
  const messageData = hashes.map(x => x.buffer);
  return createMessage(Messages.GET_TRANSACTIONS, Buffer.concat(messageData));
}

/**
 * Decode a GET_TRANSACTIONS message into the original transaction hash objects.
 * @param message - The binary message to be decoded.
 * @returns - The array of transaction hashes originally encoded into the message.
 */
export function decodeGetTransactionsRequestMessage(message: Buffer) {
  // for the time being this payload is effectively the same as the POOLED_TRANSACTION_HASHES message
  return decodeTransactionHashesMessage(message);
}

/**
 * Decode the message type from a received message.
 * @param message - The received message.
 * @returns The decoded MessageType.
 */
export function decodeMessageType(message: Buffer) {
  return message.readUInt32BE(0);
}

/**
 * Return the encoded message (minus the header) from received message buffer.
 * @param message - The complete received message.
 * @returns The encoded message, without the header.
 */
export function getEncodedMessage(message: Buffer) {
  return message.subarray(4);
}

/**
 * Creates a tx 'message' for sending to a peer.
 * @param tx - The transaction to convert to a message.
 * @returns - The message.
 */
export function toTxMessage(tx: Tx): Buffer {
  // eslint-disable-next-line jsdoc/require-jsdoc
  const createMessageComponent = (obj?: { toBuffer: () => Buffer }) => {
    if (!obj) {
      // specify a length of 0 bytes
      return numToUInt32BE(0);
    }
    const buffer = obj.toBuffer();
    return Buffer.concat([numToUInt32BE(buffer.length), buffer]);
  };
  // eslint-disable-next-line jsdoc/require-jsdoc
  const createMessageComponents = (obj?: { toBuffer: () => Buffer }[]) => {
    if (!obj || !obj.length) {
      // specify a length of 0 bytes
      return numToUInt32BE(0);
    }
    const allComponents = Buffer.concat(obj.map(createMessageComponent));
    return Buffer.concat([numToUInt32BE(obj.length), allComponents]);
  };
  const messageBuffer = Buffer.concat([
    createMessageComponent(tx.data),
    createMessageComponent(tx.proof),
    createMessageComponent(tx.encryptedLogs),
    createMessageComponent(tx.unencryptedLogs),
    createMessageComponents(tx.enqueuedPublicFunctionCalls),
    createMessageComponents(tx.newContracts),
  ]);
  const messageLength = numToUInt32BE(messageBuffer.length);
  return Buffer.concat([messageLength, messageBuffer]);
}

/**
 * Reproduces a transaction from a transaction 'message'
 * @param buffer - The message buffer to convert to a tx.
 * @returns - The reproduced transaction.
 */
export function fromTxMessage(buffer: Buffer): Tx {
  // eslint-disable-next-line jsdoc/require-jsdoc
  const toObject = <T>(objectBuffer: Buffer, factory: { fromBuffer: (b: Buffer) => T }) => {
    const objectSize = objectBuffer.readUint32BE(0);
    return {
      remainingData: objectBuffer.subarray(objectSize + 4),
      obj: objectSize === 0 ? undefined : factory.fromBuffer(objectBuffer.subarray(4, objectSize + 4)),
    };
  };

  // eslint-disable-next-line jsdoc/require-jsdoc
  const toObjectArray = <T>(objectBuffer: Buffer, factory: { fromBuffer: (b: Buffer) => T }) => {
    const output: T[] = [];
    const numItems = objectBuffer.readUint32BE(0);
    let workingBuffer = objectBuffer.subarray(4);
    for (let i = 0; i < numItems; i++) {
      const obj = toObject<T>(workingBuffer, factory);
      workingBuffer = obj.remainingData;
      if (obj !== undefined) {
        output.push(obj.obj!);
      }
    }
    return {
      remainingData: workingBuffer,
      objects: output,
    };
  };
  // this is the opposite of the 'toMessage' function
  // so the first 4 bytes is the complete length, skip it
  const publicInputs = toObject(buffer.subarray(4), KernelCircuitPublicInputs);
  const proof = toObject(publicInputs.remainingData, Proof);

  const encryptedLogs = toObject(proof.remainingData, TxL2Logs);
  if (!encryptedLogs.obj) {
    encryptedLogs.obj = new TxL2Logs([]);
  }
  const unencryptedLogs = toObject(encryptedLogs.remainingData, TxL2Logs);
  if (!unencryptedLogs.obj) {
    unencryptedLogs.obj = new TxL2Logs([]);
  }

  const publicCalls = toObjectArray(unencryptedLogs.remainingData, PublicCallRequest);
  const newContracts = toObjectArray(publicCalls.remainingData, ExtendedContractData);
  return new Tx(
    publicInputs.obj!,
    proof.obj!,
    encryptedLogs.obj,
    unencryptedLogs.obj,
    publicCalls.objects,
    newContracts.objects as Tuple<ExtendedContractData, typeof MAX_NEW_CONTRACTS_PER_TX>,
  );
}
