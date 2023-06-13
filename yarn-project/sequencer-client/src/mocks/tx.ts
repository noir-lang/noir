import {
  FunctionData,
  KERNEL_PUBLIC_CALL_STACK_LENGTH,
  KernelCircuitPublicInputs,
  makeEmptyProof,
  range,
} from '@aztec/circuits.js';
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
  FunctionL2Logs,
  PrivateTx,
  PublicTx,
  SignedTxExecutionRequest,
  Tx,
  TxExecutionRequest,
  TxL2Logs,
} from '@aztec/types';
import times from 'lodash.times';

/**
 * Testing utility to create empty encrypted logs composed from a single empty log.
 */
export function makeEmptyEncryptedLogs(): TxL2Logs {
  const functionLogs = [new FunctionL2Logs([Buffer.alloc(0)])];
  return new TxL2Logs(functionLogs);
}

/**
 * Testing utility to create a tx with an empty kernel circuit output, empty proof, and empty encrypted logs.
 */
export function makeEmptyPrivateTx(): PrivateTx {
  return Tx.createPrivate(KernelCircuitPublicInputs.empty(), makeEmptyProof(), makeEmptyEncryptedLogs(), [], []);
}

/**
 * Testing utility to create a tx with gibberish kernel circuit output, random encrypted logs, and an empty proof.
 */
export function makePrivateTx(seed = 0): PrivateTx {
  return Tx.createPrivate(
    makeKernelPublicInputs(seed),
    makeEmptyProof(),
    TxL2Logs.random(2, 3),
    [],
    times(KERNEL_PUBLIC_CALL_STACK_LENGTH, makePublicCallRequest),
  );
}

/**
 * Testing utility to create a tx with a request to execute a public function.
 */
export function makePublicTx(seed = 0): PublicTx {
  return Tx.createPublic(makeSignedTxExecutionRequest(seed));
}

/**
 * Testing utility to create a signed tx execution request.
 * @param seed - Number to derive values of this object.
 * @returns A SignedTxExecutionRequest.
 */
function makeSignedTxExecutionRequest(seed: number) {
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
}
