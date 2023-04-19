import { decode, encode } from 'rlp';
import { EthTransaction } from './eth_transaction.js';
import { sign, EthSignature, recoverFromSignature } from '../eth_sign/index.js';
import { keccak256 } from '../crypto/index.js';
import { numToUInt8 } from '../serialize/index.js';

/**
 * Represents a signed Ethereum transaction object.
 * Contains the signature, message hash, and raw transaction data for a valid Ethereum transaction.
 */
export interface SignedEthTransaction {
  /**
   * The cryptographic signature of the transaction.
   */
  signature: EthSignature;
  /**
   * The Keccak-256 hash of the signed transaction message.
   */
  messageHash: Buffer;
  /**
   * The serialized raw Ethereum transaction in RLP-encoded format.
   */
  rawTransaction: Buffer;
}

/**
 * Recover the sender's address from a raw Ethereum transaction buffer.
 * This function takes in a raw transaction buffer, extracts the signature,
 * and uses it to recover the Ethereum address of the sender. It works with
 * EIP-1559 transactions only (transaction type 2). Throws an error if there's
 * any issue during the recovery process.
 *
 * @param rawTx - The raw transaction buffer containing nonce, gas, max fees,
 *                chainId, and signature information.
 * @returns An Ethereum address as Buffer representing the transaction sender.
 */
export function recoverTransaction(rawTx: Buffer) {
  const txType = numToUInt8(rawTx[0]);
  // Slice off txType.
  const values = decode(new Uint8Array(rawTx.slice(1)));
  const v = values[9][0] || 0;
  const r = Buffer.from(values[10] as Uint8Array);
  const s = Buffer.from(values[11] as Uint8Array);
  const signature = new EthSignature(r, s, v);
  const signingDataBuf = Buffer.from(encode(values.slice(0, 9)));
  const messageHash = keccak256(Buffer.concat([txType, signingDataBuf]));
  return recoverFromSignature(messageHash, signature);
}

/**
 * Transaction type 2 (EIP1559 as per https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1559.md).
 */
export function signTransaction(tx: EthTransaction, privateKey: Buffer): SignedEthTransaction {
  const { nonce, gas, maxFeePerGas, maxPriorityFeePerGas, chainId } = tx;
  if (nonce < 0 || gas < 0 || maxFeePerGas < 0 || maxPriorityFeePerGas < 0 || chainId < 0) {
    throw new Error('negative input value.');
  }

  const txType = numToUInt8(0x2);

  const toEncode = [
    tx.chainId,
    tx.nonce,
    tx.maxPriorityFeePerGas,
    tx.maxFeePerGas,
    tx.gas,
    new Uint8Array(tx.to ? tx.to.toBuffer() : []),
    tx.value,
    new Uint8Array(tx.data || []),
    [], // access_list
  ];

  const rlpEncoded = Buffer.from(encode(toEncode));
  const messageHash = keccak256(Buffer.concat([txType, rlpEncoded]));
  const signature = sign(messageHash, privateKey);
  const { v, r, s } = signature;

  const rawTx = [...toEncode, v, r, s];
  const rawTransaction = Buffer.concat([txType, Buffer.from(encode(rawTx))]);

  return {
    signature,
    messageHash,
    rawTransaction,
  };
}

/**
 * Constructs a signed Ethereum transaction object from the given EthTransaction and EthSignature objects.
 * This function calculates the message hash by concatenating transaction type (EIP-1559) and RLP-encoded
 * transaction data, then hashing it using keccak256. The sender's address is recovered from the signature
 * and message hash using the 'recoverFromSignature' function.
 *
 * @param tx - An EthTransaction object containing transaction details such as nonce, gas, maxFeePerGas, etc.
 * @param signature - An EthSignature object containing the signature components (r, s, and v) for the transaction.
 * @returns The sender's Ethereum address recovered from the given signature and transaction data.
 */
export function signedTransaction(tx: EthTransaction, signature: EthSignature) {
  const txType = numToUInt8(0x2);

  const toEncode = [
    tx.chainId,
    tx.nonce,
    tx.maxPriorityFeePerGas,
    tx.maxFeePerGas,
    tx.gas,
    new Uint8Array(tx.to ? tx.to.toBuffer() : []),
    tx.value,
    new Uint8Array(tx.data || []),
    [], // access_list
  ];

  const rlpEncoded = Buffer.from(encode(toEncode));
  const messageHash = keccak256(Buffer.concat([txType, rlpEncoded]));

  return recoverFromSignature(messageHash, signature);
}
