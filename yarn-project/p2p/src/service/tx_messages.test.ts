import { Fr, FunctionData, KERNEL_PUBLIC_CALL_STACK_LENGTH, Proof, range } from '@aztec/circuits.js';
import {
  fr,
  makeAztecAddress,
  makeEcdsaSignature,
  makeKernelPublicInputs,
  makePublicCallRequest,
  makeSelector,
  makeTxContext,
} from '@aztec/circuits.js/factories';
import {
  EncodedContractFunction,
  SignedTxExecutionRequest,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxL2Logs,
} from '@aztec/types';
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
  return Tx.createPrivate(
    makeKernelPublicInputs(),
    Proof.fromBuffer(Buffer.alloc(10, 9)),
    TxL2Logs.random(8, 2),
    encodedPublicFunctions,
    enqueuedPublicFunctionCalls,
  );
};

const makeSignedTxExecutionRequest = (seed: number) => {
  const txRequest = TxExecutionRequest.from({
    from: makeAztecAddress(seed),
    to: makeAztecAddress(seed + 0x10),
    functionData: new FunctionData(makeSelector(seed + 0x100), true, true),
    args: range(8, seed + 0x200).map(fr),
    nonce: fr(seed + 0x300),
    txContext: makeTxContext(seed + 0x400),
    chainId: fr(seed + 0x500),
  });
  return new SignedTxExecutionRequest(txRequest, makeEcdsaSignature(seed + 0x200));
};

const makePublicTx = () => {
  return Tx.createPublic(makeSignedTxExecutionRequest(1));
};

const makePublicPrivateTx = () => {
  const publicInputs = makeKernelPublicInputs(1);
  publicInputs.end.publicCallStack = [Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO];
  return Tx.createPrivatePublic(
    publicInputs,
    Proof.fromBuffer(randomBytes(512)),
    TxL2Logs.random(8, 2),
    makeSignedTxExecutionRequest(5),
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
  expect(actual.txRequest).toBeUndefined();
};

const verifyPublicTx = (actual: Tx, expected: Tx) => {
  expect(actual.data).toBeUndefined();
  expect(actual.newContractPublicFunctions).toBeUndefined();
  expect(actual.proof).toBeUndefined();
  expect(actual.encryptedLogs).toBeUndefined();
  expect(actual.txRequest!.toBuffer()).toEqual(expected.txRequest!.toBuffer());
};

const verifyPublicPrivateTx = (actual: Tx, expected: Tx) => {
  expect(actual.data!.toBuffer()).toEqual(expected.data?.toBuffer());
  expect(actual.proof).toEqual(expected.proof);
  expect(actual.encryptedLogs!.toBuffer()).toEqual(expected.encryptedLogs?.toBuffer());
  expect(actual.txRequest!.toBuffer()).toEqual(expected.txRequest!.toBuffer());
  expect(actual.newContractPublicFunctions).toBeUndefined();
};

describe('Messages', () => {
  it('Correctly serialises and deserialises a single private transaction', () => {
    const transaction = makePrivateTx();
    const message = toTxMessage(transaction);
    const decodedTransaction = fromTxMessage(message);
    verifyPrivateTx(decodedTransaction, transaction);
  });

  it('Correctly serialises and deserialises a single public transaction', () => {
    const transaction = makePublicTx();
    const message = toTxMessage(transaction);
    const decodedTransaction = fromTxMessage(message);
    verifyPublicTx(decodedTransaction, transaction);
  });

  it('Correctly serialises and deserialises a single private/public transaction', () => {
    const transaction = makePublicPrivateTx();
    const message = toTxMessage(transaction);
    const decodedTransaction = fromTxMessage(message);
    verifyPublicPrivateTx(decodedTransaction, transaction);
  });

  it('Correctly serialises and deserialises transactions messages', () => {
    const privateTransaction = makePrivateTx();
    const publicTransaction = makePublicTx();
    const publicPrivateTransaction = makePublicPrivateTx();
    const message = createTransactionsMessage([privateTransaction, publicTransaction, publicPrivateTransaction]);
    expect(decodeMessageType(message)).toBe(Messages.POOLED_TRANSACTIONS);
    const decodedTransactions = decodeTransactionsMessage(getEncodedMessage(message));
    verifyPrivateTx(decodedTransactions[0], privateTransaction);
    verifyPublicTx(decodedTransactions[1], publicTransaction);
    verifyPublicPrivateTx(decodedTransactions[2], publicPrivateTransaction);
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
