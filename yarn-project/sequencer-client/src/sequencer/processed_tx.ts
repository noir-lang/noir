import { KernelCircuitPublicInputs } from '@aztec/circuits.js';
import { PrivateTx, PublicTx, Tx, TxHash } from '@aztec/types';
import { makeEmptyPrivateTx } from '../index.js';
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
  };

/**
 * Makes a processed tx out of a private only tx that has its proof already set.
 * @param tx - source tx that doesn't need further processing
 */
export async function makeProcessedTx(tx: PrivateTx): Promise<ProcessedTx>;

/**
 * Makes a processed tx out of a tx with a public component that needs processing.
 * @param tx - source tx
 * @param kernelOutput - output of the public kernel circuit simulation for this tx
 * @param proof - proof of the public kernel circuit for this tx
 */
export async function makeProcessedTx(
  tx: PublicTx,
  kernelOutput: KernelCircuitPublicInputs,
  proof: Proof,
): Promise<ProcessedTx>;

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
  };
}

/**
 * Makes an empty tx from an empty kernel circuit public inputs.
 * @returns A processed empty tx.
 */
export async function makeEmptyProcessedTx(): Promise<ProcessedTx> {
  const emptyTx = makeEmptyPrivateTx();
  const hash = await emptyTx.getTxHash();

  return { hash, data: emptyTx.data, proof: emptyTx.proof };
}
