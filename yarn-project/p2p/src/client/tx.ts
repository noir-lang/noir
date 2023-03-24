import { L2Block } from '@aztec/archiver';
import {
  KERNEL_NEW_COMMITMENTS_LENGTH,
  KERNEL_NEW_CONTRACTS_LENGTH,
  KERNEL_NEW_NULLIFIERS_LENGTH,
  PrivateKernelPublicInputs,
} from '@aztec/circuits.js';
import { Keccak } from 'sha3';

const hash = new Keccak(256);

/**
 * The interface of an L2 transaction.
 */
export class Tx {
  private _id?: Buffer;
  constructor(private txData: PrivateKernelPublicInputs) {}

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

  get data() {
    return this.txData;
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
        tx.txData.end.newCommitments.map(x => x.toBuffer()),
        tx.txData.end.newNullifiers.map(x => x.toBuffer()),
        tx.txData.end.newContracts.map(x => x.functionTreeRoot.toBuffer()),
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
