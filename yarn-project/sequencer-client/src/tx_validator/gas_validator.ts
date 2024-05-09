import { type Tx, type TxValidator } from '@aztec/circuit-types';
import { type AztecAddress, type Fr } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';

/** Provides a view into public contract state */
export interface PublicStateSource {
  storageRead: (contractAddress: AztecAddress, slot: Fr) => Promise<Fr>;
}

export class GasTxValidator implements TxValidator<Tx> {
  #log = createDebugLogger('aztec:sequencer:tx_validator:tx_gas');
  #publicDataSource: PublicStateSource;
  #gasTokenAddress: AztecAddress;
  #requireFees: boolean;

  constructor(publicDataSource: PublicStateSource, gasTokenAddress: AztecAddress, requireFees = false) {
    this.#publicDataSource = publicDataSource;
    this.#gasTokenAddress = gasTokenAddress;
    this.#requireFees = requireFees;
  }

  async validateTxs(txs: Tx[]): Promise<[validTxs: Tx[], invalidTxs: Tx[]]> {
    const validTxs: Tx[] = [];
    const invalidTxs: Tx[] = [];

    for (const tx of txs) {
      if (await this.#validateTxFee(tx)) {
        validTxs.push(tx);
      } else {
        invalidTxs.push(tx);
      }
    }

    return [validTxs, invalidTxs];
  }

  #validateTxFee(_tx: Tx): Promise<boolean> {
    return Promise.resolve(true);

    // TODO(#5920) re-enable sequencer checks after we have fee payer in kernel outputs
  }
}
