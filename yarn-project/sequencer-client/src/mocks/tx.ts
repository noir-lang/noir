import { KERNEL_PUBLIC_CALL_STACK_LENGTH, makeEmptyProof } from '@aztec/circuits.js';
import { makeKernelPublicInputs, makePublicCallRequest } from '@aztec/circuits.js/factories';
import { FunctionL2Logs, Tx, TxL2Logs } from '@aztec/types';
import times from 'lodash.times';

/**
 * Testing utility to create empty encrypted logs composed from a single empty log.
 */
export function makeEmptyEncryptedLogs(): TxL2Logs {
  const functionLogs = [new FunctionL2Logs([Buffer.alloc(0)])];
  return new TxL2Logs(functionLogs);
}

/**
 * Testing utility to create a tx with gibberish kernel circuit output, random encrypted logs, and an empty proof.
 */
export function makeTx(seed = 0) {
  return Tx.createTx(
    makeKernelPublicInputs(seed),
    makeEmptyProof(),
    TxL2Logs.random(2, 3),
    [],
    times(KERNEL_PUBLIC_CALL_STACK_LENGTH, makePublicCallRequest),
  );
}
