import { L2Block } from '@aztec/l2-block';
import {
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  PrivateKernelPublicInputs,
  UInt8Vector,
} from '@aztec/circuits.js';
import { Keccak } from 'sha3';

const hash = new Keccak(256);

/**
 * The interface of an L2 transaction.
 */
export class Tx {
  private _id?: Buffer;

  /**
   * 
   * @param data - Tx inputs.
   * @param proof - Tx proof.
   * @param unverifiedData  - Information not needed to verify the tx (e.g. encrypted note pre-images etc.)
   */
  constructor(public readonly data: PrivateKernelPublicInputs, public readonly proof: UInt8Vector, public readonly unverifiedData: Buffer) {}

  /**
   * Construct & return transaction ID.
   * // TODO: actually construct & return tx id.
   * @returns The transaction's id.
   */
  get txId() {
    if (!this._id) {
      this._id = Tx.createTxId(this);
    }
    return this._id;
  }

  /**
   * Utility function to generate tx ID.
   * @param tx - The transaction from which to generate the id.
   * @returns A hash of the tx data that identifies the tx.
   */
  static createTxId(tx: Tx) {
    hash.reset();
    const dataToHash = Buffer.concat(
      [
        tx.data.end.newCommitments.map(x => x.toBuffer()),
        tx.data.end.newNullifiers.map(x => x.toBuffer()),
        tx.data.end.newContracts.map(x => x.functionTreeRoot.toBuffer()),
      ].flat(),
    );
    return hash.update(dataToHash).digest();
  }
}

export function createTxIds(block: L2Block) {
  hash.reset();
  let i = 0;
  const numTxs = Math.floor(block.newCommitments.length / KERNEL_NEW_COMMITMENTS_LENGTH);
  const txIds: Buffer[] = [];
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
    txIds.push(hash.update(dataToHash).digest());
    i++;
  }
  return txIds;
}
