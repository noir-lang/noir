import { ExtendedContractData, Tx, TxHash, TxL2Logs } from '@aztec/circuit-types';
import {
  Fr,
  Header,
  Proof,
  PublicAccumulatedNonRevertibleData,
  PublicAccumulatedRevertibleData,
  PublicKernelCircuitPublicInputs,
  makeEmptyProof,
} from '@aztec/circuits.js';

/**
 * Represents a tx that has been processed by the sequencer public processor,
 * so its kernel circuit public inputs are filled in.
 */
export type ProcessedTx = Pick<Tx, 'proof' | 'encryptedLogs' | 'unencryptedLogs' | 'newContracts'> & {
  /**
   * Output of the public kernel circuit for this tx.
   */
  data: PublicKernelCircuitPublicInputs;
  /**
   * Hash of the transaction.
   */
  hash: TxHash;
  /**
   * Flag indicating the tx is 'empty' meaning it's a padding tx to take us to a power of 2.
   */
  isEmpty: boolean;
};

/**
 * Represents a tx that failed to be processed by the sequencer public processor.
 */
export type FailedTx = {
  /**
   * The failing transaction.
   */
  tx: Tx;
  /**
   * The error that caused the tx to fail.
   */
  error: Error;
};

/**
 * Makes a processed tx out of source tx.
 * @param tx - Source tx.
 * @param kernelOutput - Output of the kernel circuit simulation for this tx.
 * @param proof - Proof of the kernel circuit for this tx.
 */
export function makeProcessedTx(tx: Tx, kernelOutput?: PublicKernelCircuitPublicInputs, proof?: Proof): ProcessedTx {
  return {
    hash: tx.getTxHash(),
    data:
      kernelOutput ??
      new PublicKernelCircuitPublicInputs(
        tx.data.aggregationObject,
        PublicAccumulatedNonRevertibleData.fromPrivateAccumulatedNonRevertibleData(tx.data.endNonRevertibleData),
        PublicAccumulatedRevertibleData.fromPrivateAccumulatedRevertibleData(tx.data.end),
        tx.data.constants,
        tx.data.needsSetup,
        tx.data.needsAppLogic,
        tx.data.needsTeardown,
      ),
    proof: proof ?? tx.proof,
    encryptedLogs: tx.encryptedLogs,
    unencryptedLogs: tx.unencryptedLogs,
    newContracts: tx.newContracts,
    isEmpty: false,
  };
}

/**
 * Makes an empty tx from an empty kernel circuit public inputs.
 * @returns A processed empty tx.
 */
export function makeEmptyProcessedTx(header: Header, chainId: Fr, version: Fr): ProcessedTx {
  const emptyKernelOutput = PublicKernelCircuitPublicInputs.empty();
  emptyKernelOutput.constants.historicalHeader = header;
  emptyKernelOutput.constants.txContext.chainId = chainId;
  emptyKernelOutput.constants.txContext.version = version;
  const emptyProof = makeEmptyProof();

  const hash = new TxHash(Fr.ZERO.toBuffer());
  return {
    hash,
    encryptedLogs: new TxL2Logs([]),
    unencryptedLogs: new TxL2Logs([]),
    data: emptyKernelOutput,
    proof: emptyProof,
    newContracts: [ExtendedContractData.empty()],
    isEmpty: true,
  };
}
