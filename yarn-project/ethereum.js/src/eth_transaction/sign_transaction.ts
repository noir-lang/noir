import { decode, encode } from 'rlp';
import { EthTransaction } from './eth_transaction.js';
import { sign, EthSignature, recoverFromSignature } from '../eth_sign/index.js';
import { keccak256 } from '../crypto/index.js';
import { numToUInt8 } from '../serialize/index.js';

export interface SignedEthTransaction {
  signature: EthSignature;
  messageHash: Buffer;
  rawTransaction: Buffer;
}

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
 * Transaction type 2 (EIP1559 as per https://github.com/ethereum/EIPs/blob/master/EIPS/eip-1559.md)
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
