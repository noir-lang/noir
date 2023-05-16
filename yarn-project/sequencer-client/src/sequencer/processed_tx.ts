import { CombinedHistoricTreeRoots, KernelCircuitPublicInputs, makeEmptyProof } from '@aztec/circuits.js';
import { PrivateTx, PublicTx, Tx, TxHash, UnverifiedData } from '@aztec/types';
import { Proof } from '../prover/index.js';

/**
 * Represents a tx that has been processed by the sequencer public processor,
 * so its kernel circuit public inputs are filled in.
 */
export type ProcessedTx = Pick<Tx, 'txRequest' | 'unverifiedData'> &
  Required<Pick<Tx, 'data' | 'proof'>> & {
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
export async function makeProcessedTx(tx: PrivateTx): Promise<ProcessedTx>;

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
 * Makes a processed tx out of a private or public tx.
 * @param tx - Source tx.
 * @param kernelOutput - Output of the public kernel circuit simulation for this tx if private.
 * @param proof - Proof of the public kernel circuit for this tx if private.
 */
export async function makeProcessedTx(
  tx: Tx,
  kernelOutput?: KernelCircuitPublicInputs,
  proof?: Proof,
): Promise<ProcessedTx> {
  return {
    hash: await tx.getTxHash(),
    data: kernelOutput ?? tx.data!,
    proof: proof ?? tx.proof!,
    unverifiedData: tx.unverifiedData,
    txRequest: tx.txRequest,
    isEmpty: false,
  };
}

/**
 * Makes an empty tx from an empty kernel circuit public inputs.
 * @returns A processed empty tx.
 */
export async function makeEmptyProcessedTx(historicTreeRoots: CombinedHistoricTreeRoots): Promise<ProcessedTx> {
  const emptyKernelOutput = KernelCircuitPublicInputs.empty();
  emptyKernelOutput.constants.historicTreeRoots = historicTreeRoots;
  const emptyProof = makeEmptyProof();

  // TODO: What should be the hash of an empty tx?
  const emptyTx = Tx.createPrivate(emptyKernelOutput, emptyProof, new UnverifiedData([]), [], []);
  const hash = await emptyTx.getTxHash();

  return { hash, data: emptyKernelOutput, proof: emptyProof, isEmpty: true };
}
