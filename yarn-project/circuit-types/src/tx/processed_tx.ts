import {
  EncryptedTxL2Logs,
  PublicDataWrite,
  type SimulationError,
  type Tx,
  TxEffect,
  TxHash,
  UnencryptedTxL2Logs,
} from '@aztec/circuit-types';
import {
  Fr,
  type Gas,
  type GasFees,
  type Header,
  KernelCircuitPublicInputs,
  type Proof,
  type PublicDataUpdateRequest,
  type PublicKernelCircuitPrivateInputs,
  type PublicKernelCircuitPublicInputs,
  type PublicKernelTailCircuitPrivateInputs,
  makeEmptyProof,
} from '@aztec/circuits.js';

/**
 * Used to communicate to the prover which type of circuit to prove
 */
export enum PublicKernelType {
  NON_PUBLIC,
  SETUP,
  APP_LOGIC,
  TEARDOWN,
  TAIL,
}

export type PublicKernelTailRequest = {
  type: PublicKernelType.TAIL;
  inputs: PublicKernelTailCircuitPrivateInputs;
};

export type PublicKernelNonTailRequest = {
  type: PublicKernelType.SETUP | PublicKernelType.APP_LOGIC | PublicKernelType.TEARDOWN;
  inputs: PublicKernelCircuitPrivateInputs;
};

export type PublicKernelRequest = PublicKernelTailRequest | PublicKernelNonTailRequest;

/**
 * Represents a tx that has been processed by the sequencer public processor,
 * so its kernel circuit public inputs are filled in.
 */
export type ProcessedTx = Pick<Tx, 'proof' | 'noteEncryptedLogs' | 'encryptedLogs' | 'unencryptedLogs'> & {
  /**
   * Output of the private tail or public tail kernel circuit for this tx.
   */
  data: KernelCircuitPublicInputs;
  /**
   * Hash of the transaction.
   */
  hash: TxHash;
  /**
   * Flag indicating the tx is 'empty' meaning it's a padding tx to take us to a power of 2.
   */
  isEmpty: boolean;
  /**
   * Reason the tx was reverted.
   */
  revertReason: SimulationError | undefined;
  /**
   * The collection of public kernel circuit inputs for simulation/proving
   */
  publicKernelRequests: PublicKernelRequest[];
  /**
   * Gas usage per public execution phase.
   * Doesn't account for any base costs nor DA gas used in private execution.
   */
  gasUsed: Partial<Record<PublicKernelType, Gas>>;
  /** All public data updates for this transaction, including those created or updated by the protocol, such as balance updates from fee payments. */
  finalPublicDataUpdateRequests: PublicDataUpdateRequest[];
};

export type RevertedTx = ProcessedTx & {
  data: PublicKernelCircuitPublicInputs & {
    reverted: true;
  };

  revertReason: SimulationError;
};

export function isRevertedTx(tx: ProcessedTx): tx is RevertedTx {
  return !tx.data.revertCode.isOK();
}

export function partitionReverts(txs: ProcessedTx[]): { reverted: RevertedTx[]; nonReverted: ProcessedTx[] } {
  return txs.reduce(
    ({ reverted, nonReverted }, tx) => {
      if (isRevertedTx(tx)) {
        reverted.push(tx);
      } else {
        nonReverted.push(tx);
      }
      return { reverted, nonReverted };
    },
    { reverted: [], nonReverted: [] } as ReturnType<typeof partitionReverts>,
  );
}

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
export function makeProcessedTx(
  tx: Tx,
  kernelOutput: KernelCircuitPublicInputs,
  proof: Proof,
  publicKernelRequests: PublicKernelRequest[],
  revertReason?: SimulationError,
  gasUsed: ProcessedTx['gasUsed'] = {},
  finalPublicDataUpdateRequests?: PublicDataUpdateRequest[],
): ProcessedTx {
  return {
    hash: tx.getTxHash(),
    data: kernelOutput,
    proof,
    // TODO(4712): deal with non-revertible logs here
    noteEncryptedLogs: revertReason ? EncryptedTxL2Logs.empty() : tx.noteEncryptedLogs,
    encryptedLogs: revertReason ? EncryptedTxL2Logs.empty() : tx.encryptedLogs,
    unencryptedLogs: revertReason ? UnencryptedTxL2Logs.empty() : tx.unencryptedLogs,
    isEmpty: false,
    revertReason,
    publicKernelRequests,
    gasUsed,
    finalPublicDataUpdateRequests: finalPublicDataUpdateRequests ?? kernelOutput.end.publicDataUpdateRequests,
  };
}

/**
 * Makes an empty tx from an empty kernel circuit public inputs.
 * @returns A processed empty tx.
 */
export function makeEmptyProcessedTx(header: Header, chainId: Fr, version: Fr): ProcessedTx {
  const emptyKernelOutput = KernelCircuitPublicInputs.empty();
  emptyKernelOutput.constants.historicalHeader = header;
  emptyKernelOutput.constants.txContext.chainId = chainId;
  emptyKernelOutput.constants.txContext.version = version;
  const emptyProof = makeEmptyProof();

  const hash = new TxHash(Fr.ZERO.toBuffer());
  return {
    hash,
    noteEncryptedLogs: EncryptedTxL2Logs.empty(),
    encryptedLogs: EncryptedTxL2Logs.empty(),
    unencryptedLogs: UnencryptedTxL2Logs.empty(),
    data: emptyKernelOutput,
    proof: emptyProof,
    isEmpty: true,
    revertReason: undefined,
    publicKernelRequests: [],
    gasUsed: {},
    finalPublicDataUpdateRequests: [],
  };
}

export function toTxEffect(tx: ProcessedTx, gasFees: GasFees): TxEffect {
  return new TxEffect(
    tx.data.revertCode,
    tx.data.getTransactionFee(gasFees),
    tx.data.end.newNoteHashes.filter(h => !h.isZero()),
    tx.data.end.newNullifiers.filter(h => !h.isZero()),
    tx.data.end.newL2ToL1Msgs.filter(h => !h.isZero()),
    tx.finalPublicDataUpdateRequests.map(t => new PublicDataWrite(t.leafSlot, t.newValue)).filter(h => !h.isEmpty()),
    tx.data.end.encryptedLogPreimagesLength,
    tx.data.end.unencryptedLogPreimagesLength,
    tx.noteEncryptedLogs || EncryptedTxL2Logs.empty(),
    tx.encryptedLogs || EncryptedTxL2Logs.empty(),
    tx.unencryptedLogs || UnencryptedTxL2Logs.empty(),
  );
}

function validateProcessedTxLogs(tx: ProcessedTx): void {
  const unencryptedLogs = tx.unencryptedLogs || UnencryptedTxL2Logs.empty();
  let kernelHash = tx.data.end.unencryptedLogsHash;
  let referenceHash = Fr.fromBuffer(unencryptedLogs.hash());
  if (!referenceHash.equals(kernelHash)) {
    throw new Error(
      `Unencrypted logs hash mismatch. Expected ${referenceHash.toString()}, got ${kernelHash.toString()}.
             Processed: ${JSON.stringify(unencryptedLogs.toJSON())}
             Kernel Length: ${tx.data.end.unencryptedLogPreimagesLength}`,
    );
  }
  const encryptedLogs = tx.encryptedLogs || EncryptedTxL2Logs.empty();
  kernelHash = tx.data.end.encryptedLogsHash;
  referenceHash = Fr.fromBuffer(encryptedLogs.hash());
  if (!referenceHash.equals(kernelHash)) {
    throw new Error(
      `Encrypted logs hash mismatch. Expected ${referenceHash.toString()}, got ${kernelHash.toString()}.
             Processed: ${JSON.stringify(encryptedLogs.toJSON())}`,
    );
  }
  const noteEncryptedLogs = tx.noteEncryptedLogs || EncryptedTxL2Logs.empty();
  kernelHash = tx.data.end.noteEncryptedLogsHash;
  referenceHash = Fr.fromBuffer(noteEncryptedLogs.hash(0));
  if (!referenceHash.equals(kernelHash)) {
    throw new Error(
      `Note encrypted logs hash mismatch. Expected ${referenceHash.toString()}, got ${kernelHash.toString()}.
             Processed: ${JSON.stringify(noteEncryptedLogs.toJSON())}`,
    );
  }
  let referenceLength = new Fr(encryptedLogs.getKernelLength() + noteEncryptedLogs.getKernelLength());
  let kernelLength = tx.data.end.encryptedLogPreimagesLength;
  if (!referenceLength.equals(kernelLength)) {
    throw new Error(
      `Encrypted logs length mismatch. Expected ${referenceLength.toString()}, got ${kernelLength.toString()}.
             Processed: ${JSON.stringify(encryptedLogs.toJSON())}`,
    );
  }
  referenceLength = new Fr(unencryptedLogs.getKernelLength());
  kernelLength = tx.data.end.unencryptedLogPreimagesLength;
  if (!referenceLength.equals(kernelLength)) {
    throw new Error(
      `Unencrypted logs length mismatch. Expected ${referenceLength.toString()}, got ${kernelLength.toString()}.
             Processed: ${JSON.stringify(encryptedLogs.toJSON())}`,
    );
  }
}

export function validateProcessedTx(tx: ProcessedTx): void {
  validateProcessedTxLogs(tx);
  // TODO: validate other fields
}
