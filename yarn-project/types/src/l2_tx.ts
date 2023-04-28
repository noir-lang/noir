import { Fr } from '@aztec/foundation';
import { ContractData } from './contract_data.js';
import { TxHash } from './tx_hash.js';
import { createTxHash } from './create_tx_hash.js';
import { PublicDataWrite } from './public_data_write.js';

export class L2Tx {
  constructor(
    public newCommitments: Fr[],
    public newNullifiers: Fr[],
    public newPublicDataWrites: PublicDataWrite[],
    public newContracts: Fr[],
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
