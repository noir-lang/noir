import { Tx, type TxValidator } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';

export class MetadataTxValidator implements TxValidator<Tx> {
  #log = createDebugLogger('aztec:sequencer:tx_validator:tx_metadata');
  #l1ChainId: Fr;

  constructor(l1ChainId: number | Fr) {
    this.#l1ChainId = new Fr(l1ChainId);
  }

  validateTxs(txs: Tx[]): Promise<[validTxs: Tx[], invalidTxs: Tx[]]> {
    const validTxs: Tx[] = [];
    const invalidTxs: Tx[] = [];
    for (const tx of txs) {
      if (!this.#hasCorrectChainId(tx)) {
        invalidTxs.push(tx);
        continue;
      }

      validTxs.push(tx);
    }

    return Promise.resolve([validTxs, invalidTxs]);
  }

  #hasCorrectChainId(tx: Tx): boolean {
    if (!tx.data.constants.txContext.chainId.equals(this.#l1ChainId)) {
      this.#log.warn(
        `Rejecting tx ${Tx.getHash(
          tx,
        )} because of incorrect chain ${tx.data.constants.txContext.chainId.toNumber()} != ${this.#l1ChainId.toNumber()}`,
      );
      return false;
    } else {
      return true;
    }
  }
}
