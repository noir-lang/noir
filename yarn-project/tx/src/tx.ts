import { L2Block } from '@aztec/l2-block';
import {
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  PrivateKernelPublicInputs,
  UInt8Vector,
} from '@aztec/circuits.js';
import { Keccak } from 'sha3';
import { TxHash } from './tx_hash.js';

const hash = new Keccak(256);

/**
 * The interface of an L2 transaction.
 */
export class Tx {
  private _hash?: TxHash;

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
    public readonly unverifiedData: Buffer,
    public readonly isEmpty = false,
  ) {}

  /**
   * Construct & return transaction hash.
   * @returns The transaction's hash.
   */
  get txHash() {
    if (!this._hash) {
      this._hash = Tx.createTxHash(this);
    }
    return this._hash;
  }

  /**
   * Utility function to generate tx hash.
   * @param tx - The transaction from which to generate the hash.
   * @returns A hash of the tx data that identifies the tx.
   */
  static createTxHash(tx: Tx): TxHash {
    hash.reset();
    const dataToHash = Buffer.concat(
      [
        tx.data.end.newCommitments.map(x => x.toBuffer()),
        tx.data.end.newNullifiers.map(x => x.toBuffer()),
        tx.data.end.newContracts.map(x => x.functionTreeRoot.toBuffer()),
      ].flat(),
    );
    return new TxHash(hash.update(dataToHash).digest());
  }
}

/**
 * Generates transaction hashes for the transactions in an L2 block.
 * @param block - The L2 block.
 * @returns An array of hashes, one for each tx.
 */
export function createTxHashes(block: L2Block) {
  hash.reset();
  let i = 0;
  const numTxs = Math.floor(block.newCommitments.length / KERNEL_NEW_COMMITMENTS_LENGTH);
  const txHashes: TxHash[] = [];
  while (i < numTxs) {
    const dataToHash = Buffer.concat(
      [
        block.newCommitments
          .slice(i * KERNEL_NEW_COMMITMENTS_LENGTH, i * KERNEL_NEW_COMMITMENTS_LENGTH + KERNEL_NEW_COMMITMENTS_LENGTH)
          .map(x => x.toBuffer()),
        block.newNullifiers
          .slice(i * KERNEL_NEW_NULLIFIERS_LENGTH, i * KERNEL_NEW_NULLIFIERS_LENGTH + KERNEL_NEW_NULLIFIERS_LENGTH)
          .map(x => x.toBuffer()),
        block.newContracts
          .slice(i * KERNEL_NEW_CONTRACTS_LENGTH, i * KERNEL_NEW_CONTRACTS_LENGTH + KERNEL_NEW_CONTRACTS_LENGTH)
          .map(x => x.toBuffer()),
      ].flat(),
    );
    txHashes.push(new TxHash(hash.update(dataToHash).digest()));
    i++;
  }
  return txHashes;
}
