import {
  EncryptedNoteTxL2Logs,
  EncryptedTxL2Logs,
  PublicDataWrite,
  type PublicInputsAndRecursiveProof,
  type PublicInputsAndTubeProof,
  type SimulationError,
  type Tx,
  TxEffect,
  TxHash,
  UnencryptedTxL2Logs,
} from '@aztec/circuit-types';
import {
  type AvmExecutionHints,
  ClientIvcProof,
  Fr,
  type Gas,
  type GasFees,
  type Header,
  KernelCircuitPublicInputs,
  type NESTED_RECURSIVE_PROOF_LENGTH,
  type PublicDataUpdateRequest,
  type PublicKernelCircuitPrivateInputs,
  type PublicKernelCircuitPublicInputs,
  type PublicKernelTailCircuitPrivateInputs,
  type RecursiveProof,
  type TUBE_PROOF_LENGTH,
  type VerificationKeyData,
} from '@aztec/circuits.js';

import { type CircuitName } from '../stats/stats.js';

/**
 * Used to communicate to the prover which type of circuit to prove
 */
export enum PublicKernelType {
  NON_PUBLIC = 'non-public',
  SETUP = 'setup',
  APP_LOGIC = 'app-logic',
  TEARDOWN = 'teardown',
  TAIL = 'tail',
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

export const AVM_REQUEST = 'AVM' as const;

export type AvmProvingRequest = {
  type: typeof AVM_REQUEST;
  functionName: string; // informational only
  bytecode: Buffer;
  calldata: Fr[];
  avmHints: AvmExecutionHints;
  kernelRequest: PublicKernelNonTailRequest;
};

export type PublicProvingRequest = AvmProvingRequest | PublicKernelRequest;

/**
 * Represents a tx that has been processed by the sequencer public processor,
 * so its kernel circuit public inputs are filled in.
 */
export type ProcessedTx = Pick<Tx, 'clientIvcProof' | 'noteEncryptedLogs' | 'encryptedLogs' | 'unencryptedLogs'> & {
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
   * The inputs for AVM and kernel proving.
   */
  publicProvingRequests: PublicProvingRequest[];
  /**
   * Gas usage per public execution phase.
   * Doesn't account for any base costs nor DA gas used in private execution.
   */
  gasUsed: Partial<Record<PublicKernelType, Gas>>;
  /**
   * All public data updates for this transaction, including those created
   * or updated by the protocol, such as balance updates from fee payments.
   */
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
  publicProvingRequests: PublicProvingRequest[],
  revertReason?: SimulationError,
  gasUsed: ProcessedTx['gasUsed'] = {},
  finalPublicDataUpdateRequests?: PublicDataUpdateRequest[],
): ProcessedTx {
  return {
    hash: tx.getTxHash(),
    data: kernelOutput,
    clientIvcProof: tx.clientIvcProof,
    // TODO(4712): deal with non-revertible logs here
    noteEncryptedLogs: tx.noteEncryptedLogs,
    encryptedLogs: tx.encryptedLogs,
    unencryptedLogs: tx.unencryptedLogs,
    isEmpty: false,
    revertReason,
    publicProvingRequests,
    gasUsed,
    finalPublicDataUpdateRequests: finalPublicDataUpdateRequests ?? kernelOutput.end.publicDataUpdateRequests,
  };
}

export type PaddingProcessedTx = ProcessedTx & {
  verificationKey: VerificationKeyData;
  recursiveProof: RecursiveProof<typeof NESTED_RECURSIVE_PROOF_LENGTH>;
};

export type PaddingProcessedTxFromTube = ProcessedTx & {
  verificationKey: VerificationKeyData;
  recursiveProof: RecursiveProof<typeof TUBE_PROOF_LENGTH>;
};

/**
 * Makes a padding empty tx with a valid proof.
 * @returns A valid padding processed tx.
 */
export function makePaddingProcessedTx(
  kernelOutput: PublicInputsAndRecursiveProof<KernelCircuitPublicInputs>,
): PaddingProcessedTx {
  const hash = new TxHash(Fr.ZERO.toBuffer());
  return {
    hash,
    noteEncryptedLogs: EncryptedNoteTxL2Logs.empty(),
    encryptedLogs: EncryptedTxL2Logs.empty(),
    unencryptedLogs: UnencryptedTxL2Logs.empty(),
    data: kernelOutput.inputs,
    clientIvcProof: ClientIvcProof.empty(),
    isEmpty: true,
    revertReason: undefined,
    publicProvingRequests: [],
    gasUsed: {},
    finalPublicDataUpdateRequests: [],
    verificationKey: kernelOutput.verificationKey,
    recursiveProof: kernelOutput.proof,
  };
}

/**
 * Makes a padding empty tx with a valid proof.
 * @returns A valid padding processed tx.
 */
export function makePaddingProcessedTxFromTubeProof(
  kernelOutput: PublicInputsAndTubeProof<KernelCircuitPublicInputs>,
): PaddingProcessedTxFromTube {
  const hash = new TxHash(Fr.ZERO.toBuffer());
  return {
    hash,
    noteEncryptedLogs: EncryptedNoteTxL2Logs.empty(),
    encryptedLogs: EncryptedTxL2Logs.empty(),
    unencryptedLogs: UnencryptedTxL2Logs.empty(),
    data: kernelOutput.inputs,
    clientIvcProof: ClientIvcProof.empty(),
    isEmpty: true,
    revertReason: undefined,
    publicProvingRequests: [],
    gasUsed: {},
    finalPublicDataUpdateRequests: [],
    verificationKey: kernelOutput.verificationKey,
    recursiveProof: kernelOutput.proof,
  };
}

/**
 * Makes an empty tx from an empty kernel circuit public inputs.
 * @returns A processed empty tx.
 */
export function makeEmptyProcessedTx(header: Header, chainId: Fr, version: Fr, vkTreeRoot: Fr): ProcessedTx {
  const emptyKernelOutput = KernelCircuitPublicInputs.empty();
  emptyKernelOutput.constants.historicalHeader = header;
  emptyKernelOutput.constants.txContext.chainId = chainId;
  emptyKernelOutput.constants.txContext.version = version;
  emptyKernelOutput.constants.vkTreeRoot = vkTreeRoot;

  const hash = new TxHash(Fr.ZERO.toBuffer());
  return {
    hash,
    noteEncryptedLogs: EncryptedNoteTxL2Logs.empty(),
    encryptedLogs: EncryptedTxL2Logs.empty(),
    unencryptedLogs: UnencryptedTxL2Logs.empty(),
    data: emptyKernelOutput,
    clientIvcProof: ClientIvcProof.empty(),
    isEmpty: true,
    revertReason: undefined,
    publicProvingRequests: [],
    gasUsed: {},
    finalPublicDataUpdateRequests: [],
  };
}

export function toTxEffect(tx: ProcessedTx, gasFees: GasFees): TxEffect {
  return new TxEffect(
    tx.data.revertCode,
    tx.data.getTransactionFee(gasFees),
    tx.data.end.noteHashes.filter(h => !h.isZero()),
    tx.data.end.nullifiers.filter(h => !h.isZero()),
    tx.data.end.l2ToL1Msgs.filter(h => !h.isZero()),
    tx.finalPublicDataUpdateRequests.map(t => new PublicDataWrite(t.leafSlot, t.newValue)).filter(h => !h.isEmpty()),
    tx.data.end.noteEncryptedLogPreimagesLength,
    tx.data.end.encryptedLogPreimagesLength,
    tx.data.end.unencryptedLogPreimagesLength,
    tx.noteEncryptedLogs || EncryptedNoteTxL2Logs.empty(),
    tx.encryptedLogs || EncryptedTxL2Logs.empty(),
    tx.unencryptedLogs || UnencryptedTxL2Logs.empty(),
  );
}

function validateProcessedTxLogs(tx: ProcessedTx): void {
  const noteEncryptedLogs = tx.noteEncryptedLogs || EncryptedNoteTxL2Logs.empty();
  let kernelHash = tx.data.end.noteEncryptedLogsHash;
  let referenceHash = Fr.fromBuffer(noteEncryptedLogs.hash());
  if (!referenceHash.equals(kernelHash)) {
    throw new Error(
      `Note encrypted logs hash mismatch. Expected ${referenceHash.toString()}, got ${kernelHash.toString()}.
             Processed: ${JSON.stringify(noteEncryptedLogs.toJSON())}`,
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
  const unencryptedLogs = tx.unencryptedLogs || UnencryptedTxL2Logs.empty();
  kernelHash = Fr.fromBuffer(
    UnencryptedTxL2Logs.hashSiloedLogs(
      tx.data.end.unencryptedLogsHashes.filter(hash => !hash.isEmpty()).map(h => h.getSiloedHash()),
    ),
  );
  referenceHash = Fr.fromBuffer(unencryptedLogs.hash());
  if (!referenceHash.equals(kernelHash)) {
    throw new Error(
      `Unencrypted logs hash mismatch. Expected ${referenceHash.toString()}, got ${kernelHash.toString()}.
             Processed: ${JSON.stringify(unencryptedLogs.toJSON())}
             Kernel Length: ${tx.data.end.unencryptedLogPreimagesLength}`,
    );
  }
  let referenceLength = new Fr(noteEncryptedLogs.getKernelLength());
  let kernelLength = tx.data.end.noteEncryptedLogPreimagesLength;
  if (!referenceLength.equals(kernelLength)) {
    throw new Error(
      `Note encrypted logs length mismatch. Expected ${referenceLength.toString()}, got ${kernelLength.toString()}.
             Processed: ${JSON.stringify(noteEncryptedLogs.toJSON())}`,
    );
  }
  referenceLength = new Fr(encryptedLogs.getKernelLength());
  kernelLength = tx.data.end.encryptedLogPreimagesLength;
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
             Processed: ${JSON.stringify(unencryptedLogs.toJSON())}`,
    );
  }
}

export function validateProcessedTx(tx: ProcessedTx): void {
  validateProcessedTxLogs(tx);
  // TODO: validate other fields
}

export function mapPublicKernelToCircuitName(kernelType: PublicKernelRequest['type']): CircuitName {
  switch (kernelType) {
    case PublicKernelType.SETUP:
      return 'public-kernel-setup';
    case PublicKernelType.APP_LOGIC:
      return 'public-kernel-app-logic';
    case PublicKernelType.TEARDOWN:
      return 'public-kernel-teardown';
    case PublicKernelType.TAIL:
      return 'public-kernel-tail';
    default:
      throw new Error(`Unknown kernel type: ${kernelType}`);
  }
}
