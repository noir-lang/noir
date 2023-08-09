import { ConstantHistoricBlockData, Fr, KernelCircuitPublicInputs, Proof, makeEmptyProof } from '@aztec/circuits.js';
import { Tx, TxHash, TxL2Logs } from '@aztec/types';

/**
 * Represents a tx that has been processed by the sequencer public processor,
 * so its kernel circuit public inputs are filled in.
 */
export type ProcessedTx = Pick<Tx, 'data' | 'proof' | 'encryptedLogs' | 'unencryptedLogs'> & {
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
 * Makes a processed tx out of a private only tx that has its proof already set.
 * @param tx - Source tx that doesn't need further processing.
 */
export async function makeProcessedTx(tx: Tx): Promise<ProcessedTx>;

/**
 * Makes a processed tx out of a tx with a public component that needs processing.
 * @param tx - Source tx.
 * @param kernelOutput - Output of the public kernel circuit simulation for this tx.
 * @param proof - Proof of the public kernel circuit for this tx.
 */
export async function makeProcessedTx(
  tx: Tx,
  kernelOutput: KernelCircuitPublicInputs,
  proof: Proof,
): Promise<ProcessedTx>;

/**
 * Makes a processed tx out of source tx.
 * @param tx - Source tx.
 * @param kernelOutput - Output of the kernel circuit simulation for this tx.
 * @param proof - Proof of the kernel circuit for this tx.
 */
export async function makeProcessedTx(
  tx: Tx,
  kernelOutput?: KernelCircuitPublicInputs,
  proof?: Proof,
): Promise<ProcessedTx> {
  return {
    hash: await tx.getTxHash(),
    data: kernelOutput ?? tx.data,
    proof: proof ?? tx.proof,
    encryptedLogs: tx.encryptedLogs,
    unencryptedLogs: tx.unencryptedLogs,
    isEmpty: false,
  };
}

/**
 * Makes an empty tx from an empty kernel circuit public inputs.
 * @returns A processed empty tx.
 */
export function makeEmptyProcessedTx(
  historicTreeRoots: ConstantHistoricBlockData,
  chainId: Fr,
  version: Fr,
): Promise<ProcessedTx> {
  const emptyKernelOutput = KernelCircuitPublicInputs.empty();
  emptyKernelOutput.constants.blockData = historicTreeRoots;
  emptyKernelOutput.constants.txContext.chainId = chainId;
  emptyKernelOutput.constants.txContext.version = version;
  const emptyProof = makeEmptyProof();

  const hash = new TxHash(Fr.ZERO.toBuffer());
  return Promise.resolve({
    hash,
    encryptedLogs: new TxL2Logs([]),
    unencryptedLogs: new TxL2Logs([]),
    data: emptyKernelOutput,
    proof: emptyProof,
    isEmpty: true,
  });
}
