import { type AnyTx, Tx, type TxValidator } from '@aztec/circuit-types';
import { type GlobalVariables } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';

export class MetadataTxValidator<T extends AnyTx> implements TxValidator<T> {
  #log = createDebugLogger('aztec:sequencer:tx_validator:tx_metadata');
  #globalVariables: GlobalVariables;

  constructor(globalVariables: GlobalVariables) {
    this.#globalVariables = globalVariables;
  }

  validateTxs(txs: T[]): Promise<[validTxs: T[], invalidTxs: T[]]> {
    const validTxs: T[] = [];
    const invalidTxs: T[] = [];
    for (const tx of txs) {
      if (!this.#hasCorrectChainId(tx)) {
        invalidTxs.push(tx);
        continue;
      }

      if (!this.#isValidForBlockNumber(tx)) {
        invalidTxs.push(tx);
        continue;
      }

      validTxs.push(tx);
    }

    return Promise.resolve([validTxs, invalidTxs]);
  }

  #hasCorrectChainId(tx: T): boolean {
    if (!tx.data.constants.txContext.chainId.equals(this.#globalVariables.chainId)) {
      this.#log.warn(
        `Rejecting tx ${Tx.getHash(
          tx,
        )} because of incorrect chain ${tx.data.constants.txContext.chainId.toNumber()} != ${this.#globalVariables.chainId.toNumber()}`,
      );
      return false;
    } else {
      return true;
    }
  }

  #isValidForBlockNumber(tx: T): boolean {
    const target =
      tx instanceof Tx
        ? tx.data.forRollup?.rollupValidationRequests || tx.data.forPublic!.validationRequests.forRollup
        : tx.data.rollupValidationRequests;
    const maxBlockNumber = target.maxBlockNumber;

    if (maxBlockNumber.isSome && maxBlockNumber.value < this.#globalVariables.blockNumber) {
      this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} for low max block number`);
      return false;
    } else {
      return true;
    }
  }
}
