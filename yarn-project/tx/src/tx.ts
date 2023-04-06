import { PrivateKernelPublicInputs, UInt8Vector } from '@aztec/circuits.js';
import { serializeToBuffer } from '@aztec/circuits.js/utils';
import { keccak } from '@aztec/foundation';
import { UnverifiedData } from '@aztec/unverified-data';
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
        // Keep this line in sync with newContractData from createTxHashes
        tx.data.end.newContracts.map(x => serializeToBuffer(x.contractAddress, x.portalContractAddress)),
      ].flat(),
    );
    return new TxHash(keccak(dataToHash));
  }
}
