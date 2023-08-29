import { Tx, TxHash, mockTx } from '@aztec/types';

import { expect } from '@jest/globals';
import { randomBytes } from 'crypto';

import {
  Messages,
  createGetTransactionsRequestMessage,
  createTransactionHashesMessage,
  createTransactionsMessage,
  decodeGetTransactionsRequestMessage,
  decodeMessageType,
  decodeTransactionHashesMessage,
  decodeTransactionsMessage,
  fromTxMessage,
  getEncodedMessage,
  toTxMessage,
} from './tx_messages.js';

const makeTxHash = () => {
  return new TxHash(randomBytes(32));
};

const verifyTx = (actual: Tx, expected: Tx) => {
  expect(actual.data!.toBuffer()).toEqual(expected.data?.toBuffer());
  expect(actual.proof!.toBuffer()).toEqual(expected.proof!.toBuffer());
  expect(actual.encryptedLogs!.toBuffer()).toEqual(expected.encryptedLogs?.toBuffer());
  expect(actual.newContracts!.length).toEqual(expected.newContracts!.length);
  for (let i = 0; i < actual.newContracts!.length; i++) {
    expect(actual.newContracts![i].toBuffer()).toEqual(expected.newContracts![i].toBuffer());
  }
};

describe('Messages', () => {
  it('Correctly serialises and deserialises a single private transaction', () => {
    const transaction = mockTx();
    const message = toTxMessage(transaction);
    const decodedTransaction = fromTxMessage(message);
    verifyTx(decodedTransaction, transaction);
  });

  it('Correctly serialises and deserialises transactions messages', () => {
    const privateTransactions = [mockTx(), mockTx(), mockTx()];
    const message = createTransactionsMessage(privateTransactions);
    expect(decodeMessageType(message)).toBe(Messages.POOLED_TRANSACTIONS);
    const decodedTransactions = decodeTransactionsMessage(getEncodedMessage(message));
    verifyTx(decodedTransactions[0], privateTransactions[0]);
    verifyTx(decodedTransactions[1], privateTransactions[1]);
    verifyTx(decodedTransactions[2], privateTransactions[2]);
  });

  it('Correctly serialises and deserialises transaction hashes message', () => {
    const txHashes = [makeTxHash(), makeTxHash(), makeTxHash()];
    const message = createTransactionHashesMessage(txHashes);
    expect(decodeMessageType(message)).toEqual(Messages.POOLED_TRANSACTION_HASHES);
    const decodedHashes = decodeTransactionHashesMessage(getEncodedMessage(message));
    expect(decodedHashes.map(x => x.toString())).toEqual(txHashes.map(x => x.toString()));
  });

  it('Correctly serialises and deserialises get transactions message', () => {
    const txHashes = [makeTxHash(), makeTxHash(), makeTxHash()];
    const message = createGetTransactionsRequestMessage(txHashes);
    expect(decodeMessageType(message)).toEqual(Messages.GET_TRANSACTIONS);
    const decodedHashes = decodeGetTransactionsRequestMessage(getEncodedMessage(message));
    expect(decodedHashes.map(x => x.toString())).toEqual(txHashes.map(x => x.toString()));
  });
});
