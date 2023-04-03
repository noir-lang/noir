import {
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  PrivateKernelPublicInputs,
  UInt8Vector,
} from '@aztec/circuits.js';
import { keccak } from '@aztec/foundation';
import { L2Block, UnverifiedData } from '@aztec/l2-block';
import { TxHash } from './tx_hash.js';

/**
 * The interface of an L2 transaction.
 */
export class Tx {
  /**
   *
   * @param data - Tx inputs.
   * @param proof - Tx proof.
   * @param unverifiedData  - Information not needed to verify the tx (e.g. encrypted note pre-images etc.)
   * @param isEmpty - Whether this is a placeholder empty tx.
   */
  constructor(
    public readonly data: PrivateKernelPublicInputs,
    public readonly proof: UInt8Vector,
    public readonly unverifiedData: UnverifiedData,
    private hash?: TxHash,
    public readonly isEmpty = false,
  ) {}

  /**
   * Construct & return transaction hash.
   * @returns The transaction's hash.
   */
  get txHash() {
    if (!this.hash) {
      this.hash = Tx.createTxHash(this);
    }
    return this.hash;
  }

  /**
   * Utility function to generate tx hash.
   * @param tx - The transaction from which to generate the hash.
   * @returns A hash of the tx data that identifies the tx.
   */
  static createTxHash(tx: Tx): TxHash {
    const dataToHash = Buffer.concat(
      [
        tx.data.end.newCommitments.map(x => x.toBuffer()),
        tx.data.end.newNullifiers.map(x => x.toBuffer()),
        tx.data.end.newContracts.map(x => x.functionTreeRoot.toBuffer()),
      ].flat(),
    );
    return new TxHash(keccak(dataToHash));
  }
}

/**
 * Generates transaction hash for the ith transaction in an L2 block.
 * @param block - The L2 block.
 * @param txIndex - The index of the tx in the block.
 * @returns TxHash of the tx.
 */
export function getTxHash(block: L2Block, txIndex: number) {
  const dataToHash = Buffer.concat(
    [
      block.newCommitments
        .slice(
          txIndex * KERNEL_NEW_COMMITMENTS_LENGTH,
          txIndex * KERNEL_NEW_COMMITMENTS_LENGTH + KERNEL_NEW_COMMITMENTS_LENGTH,
        )
        .map(x => x.toBuffer()),
      block.newNullifiers
        .slice(
          txIndex * KERNEL_NEW_NULLIFIERS_LENGTH,
          txIndex * KERNEL_NEW_NULLIFIERS_LENGTH + KERNEL_NEW_NULLIFIERS_LENGTH,
        )
        .map(x => x.toBuffer()),
      block.newContracts
        .slice(
          txIndex * KERNEL_NEW_CONTRACTS_LENGTH,
          txIndex * KERNEL_NEW_CONTRACTS_LENGTH + KERNEL_NEW_CONTRACTS_LENGTH,
        )
        .map(x => x.toBuffer()),
    ].flat(),
  );
  return new TxHash(keccak(dataToHash));
}

/**
 * Generates transaction hashes for the transactions in an L2 block.
 * @param block - The L2 block.
 * @returns An array of hashes, one for each tx.
 */
export function createTxHashes(block: L2Block) {
  const numTxs = Math.floor(block.newCommitments.length / KERNEL_NEW_COMMITMENTS_LENGTH);
  return Array(numTxs)
    .fill(0)
    .map((_, i) => getTxHash(block, i));
}
