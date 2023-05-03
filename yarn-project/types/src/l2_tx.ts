import { Fr } from '@aztec/foundation/fields';
import { ContractData } from './contract_data.js';
import { TxHash } from './tx_hash.js';
import { createTxHash } from './create_tx_hash.js';
import { PublicDataWrite } from './public_data_write.js';

/**
 * Represents an L2 transaction.
 */
export class L2Tx {
  constructor(
    /**
     * New commitments created by the transaction.
     */
    public newCommitments: Fr[],
    /**
     * New nullifiers created by the transaction.
     */
    public newNullifiers: Fr[],
    /**
     * New public data writes created by the transaction.
     */
    public newPublicDataWrites: PublicDataWrite[],
    /**
     * New contracts leafs created by the transaction to be inserted into the contract tree.
     */
    public newContracts: Fr[],
    /**
     * New contract data created by the transaction.
     */
    public newContractData: ContractData[],
    private hash?: TxHash,
  ) {}

  /**
   * Construct & return transaction hash.
   * @returns The transaction's hash.
   */
  get txHash() {
    if (!this.hash) {
      this.hash = createTxHash(this);
    }
    return this.hash;
  }
}
