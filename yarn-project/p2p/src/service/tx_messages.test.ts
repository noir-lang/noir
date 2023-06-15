import { KERNEL_PUBLIC_CALL_STACK_LENGTH, Proof } from '@aztec/circuits.js';
import { makeKernelPublicInputs, makePublicCallRequest } from '@aztec/circuits.js/factories';
import { EncodedContractFunction, Tx, TxHash, TxL2Logs } from '@aztec/types';
import { expect } from '@jest/globals';
import { randomBytes } from 'crypto';
import times from 'lodash.times';
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

const makePrivateTx = () => {
  const encodedPublicFunctions = [EncodedContractFunction.random(), EncodedContractFunction.random()];
  const enqueuedPublicFunctionCalls = times(KERNEL_PUBLIC_CALL_STACK_LENGTH, i => makePublicCallRequest(i));
  return Tx.createTx(
    makeKernelPublicInputs(),
    Proof.fromBuffer(Buffer.alloc(10, 9)),
    TxL2Logs.random(8, 2),
    encodedPublicFunctions,
    enqueuedPublicFunctionCalls,
  );
};

const makeTxHash = () => {
  return new TxHash(randomBytes(32));
};

const verifyPrivateTx = (actual: Tx, expected: Tx) => {
  expect(actual.data!.toBuffer()).toEqual(expected.data?.toBuffer());
  expect(actual.proof!.toBuffer()).toEqual(expected.proof!.toBuffer());
  expect(actual.encryptedLogs!.toBuffer()).toEqual(expected.encryptedLogs?.toBuffer());
  expect(actual.newContractPublicFunctions!.length).toEqual(expected.newContractPublicFunctions!.length);
  for (let i = 0; i < actual.newContractPublicFunctions!.length; i++) {
    expect(actual.newContractPublicFunctions![i].toBuffer()).toEqual(
      expected.newContractPublicFunctions![i].toBuffer(),
    );
  }
};

describe('Messages', () => {
  it('Correctly serialises and deserialises a single private transaction', () => {
    const transaction = makePrivateTx();
    const message = toTxMessage(transaction);
    const decodedTransaction = fromTxMessage(message);
    verifyPrivateTx(decodedTransaction, transaction);
  });

  it('Correctly serialises and deserialises transactions messages', () => {
    const privateTransactions = [makePrivateTx(), makePrivateTx(), makePrivateTx()];
    const message = createTransactionsMessage(privateTransactions);
    expect(decodeMessageType(message)).toBe(Messages.POOLED_TRANSACTIONS);
    const decodedTransactions = decodeTransactionsMessage(getEncodedMessage(message));
    verifyPrivateTx(decodedTransactions[0], privateTransactions[0]);
    verifyPrivateTx(decodedTransactions[1], privateTransactions[1]);
    verifyPrivateTx(decodedTransactions[2], privateTransactions[2]);
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
