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
  type Header,
  KernelCircuitPublicInputs,
  type Proof,
  type PublicKernelCircuitPublicInputs,
  type SideEffect,
  type SideEffectLinkedToNoteHash,
  makeEmptyProof,
} from '@aztec/circuits.js';

/**
 * Represents a tx that has been processed by the sequencer public processor,
 * so its kernel circuit public inputs are filled in.
 */
export type ProcessedTx = Pick<Tx, 'proof' | 'encryptedLogs' | 'unencryptedLogs'> & {
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
  revertReason?: SimulationError,
): ProcessedTx {
  return {
    hash: tx.getTxHash(),
    data: kernelOutput,
    proof,
    encryptedLogs: revertReason ? EncryptedTxL2Logs.empty() : tx.encryptedLogs,
    unencryptedLogs: revertReason ? UnencryptedTxL2Logs.empty() : tx.unencryptedLogs,
    isEmpty: false,
    revertReason,
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
    encryptedLogs: EncryptedTxL2Logs.empty(),
    unencryptedLogs: UnencryptedTxL2Logs.empty(),
    data: emptyKernelOutput,
    proof: emptyProof,
    isEmpty: true,
    revertReason: undefined,
  };
}

export function toTxEffect(tx: ProcessedTx): TxEffect {
  return new TxEffect(
    tx.data.revertCode,
    tx.data.end.newNoteHashes.map((c: SideEffect) => c.value).filter(h => !h.isZero()),
    tx.data.end.newNullifiers.map((n: SideEffectLinkedToNoteHash) => n.value).filter(h => !h.isZero()),
    tx.data.end.newL2ToL1Msgs.filter(h => !h.isZero()),
    tx.data.end.publicDataUpdateRequests
      .map(t => new PublicDataWrite(t.leafSlot, t.newValue))
      .filter(h => !h.isEmpty()),
    tx.encryptedLogs || EncryptedTxL2Logs.empty(),
    tx.unencryptedLogs || UnencryptedTxL2Logs.empty(),
  );
}

function validateProcessedTxLogs(tx: ProcessedTx): void {
  const unencryptedLogs = tx.unencryptedLogs || UnencryptedTxL2Logs.empty();
  const kernelUnencryptedLogsHash = tx.data.end.unencryptedLogsHash;
  const referenceHash = Fr.fromBuffer(unencryptedLogs.hash());
  if (!referenceHash.equals(kernelUnencryptedLogsHash)) {
    throw new Error(
      `Unencrypted logs hash mismatch. Expected ${referenceHash.toString()}, got ${kernelUnencryptedLogsHash.toString()}.
             Processed: ${JSON.stringify(unencryptedLogs.toJSON())}
             Kernel Length: ${tx.data.end.unencryptedLogPreimagesLength}`,
    );
  }
}

export function validateProcessedTx(tx: ProcessedTx): void {
  validateProcessedTxLogs(tx);
  // TODO: validate other fields
}
