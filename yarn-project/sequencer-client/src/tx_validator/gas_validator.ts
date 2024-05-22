import { type Tx, type TxValidator } from '@aztec/circuit-types';
import { type AztecAddress, type Fr } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { computeFeePayerBalanceStorageSlot } from '@aztec/simulator';

/** Provides a view into public contract state */
export interface PublicStateSource {
  storageRead: (contractAddress: AztecAddress, slot: Fr) => Promise<Fr>;
}

export class GasTxValidator implements TxValidator<Tx> {
  #log = createDebugLogger('aztec:sequencer:tx_validator:tx_gas');
  #publicDataSource: PublicStateSource;
  #gasTokenAddress: AztecAddress;

  constructor(publicDataSource: PublicStateSource, gasTokenAddress: AztecAddress) {
    this.#publicDataSource = publicDataSource;
    this.#gasTokenAddress = gasTokenAddress;
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

  async #validateTxFee(tx: Tx): Promise<boolean> {
    const feePayer = tx.data.feePayer;
    // TODO(@spalladino) Eventually remove the is_zero condition as we should always charge fees to every tx
    if (feePayer.isZero()) {
      return true;
    }
    const feeLimit = tx.data.constants.txContext.gasSettings.getFeeLimit();
    const balance = await this.#publicDataSource.storageRead(
      this.#gasTokenAddress,
      computeFeePayerBalanceStorageSlot(feePayer),
    );
    if (balance.lt(feeLimit)) {
      this.#log.info(`Rejecting transaction due to not enough fee payer balance`, { feePayer, balance, feeLimit });
      return false;
    }
    return true;
  }
}
