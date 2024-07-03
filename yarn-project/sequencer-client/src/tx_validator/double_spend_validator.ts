import { type AnyTx, Tx, type TxValidator } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';

export interface NullifierSource {
  getNullifierIndex: (nullifier: Fr) => Promise<bigint | undefined>;
}

export class DoubleSpendTxValidator<T extends AnyTx> implements TxValidator<T> {
  #log = createDebugLogger('aztec:sequencer:tx_validator:tx_double_spend');
  #nullifierSource: NullifierSource;

  constructor(nullifierSource: NullifierSource) {
    this.#nullifierSource = nullifierSource;
  }

  async validateTxs(txs: T[]): Promise<[validTxs: T[], invalidTxs: T[]]> {
    const validTxs: T[] = [];
    const invalidTxs: T[] = [];
    const thisBlockNullifiers = new Set<bigint>();

    for (const tx of txs) {
      if (!(await this.#uniqueNullifiers(tx, thisBlockNullifiers))) {
        invalidTxs.push(tx);
        continue;
      }

      validTxs.push(tx);
    }

    return [validTxs, invalidTxs];
  }

  async #uniqueNullifiers(tx: AnyTx, thisBlockNullifiers: Set<bigint>): Promise<boolean> {
    const nullifiers = tx.data.getNonEmptyNullifiers().map(x => x.toBigInt());

    // Ditch this tx if it has repeated nullifiers
    const uniqueNullifiers = new Set(nullifiers);
    if (uniqueNullifiers.size !== nullifiers.length) {
      this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} for emitting duplicate nullifiers`);
      return false;
    }

    for (const nullifier of nullifiers) {
      if (thisBlockNullifiers.has(nullifier)) {
        this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} for repeating a nullifier in the same block`);
        return false;
      }

      thisBlockNullifiers.add(nullifier);
    }

    const nullifierIndexes = await Promise.all(nullifiers.map(n => this.#nullifierSource.getNullifierIndex(new Fr(n))));

    const hasDuplicates = nullifierIndexes.some(index => index !== undefined);
    if (hasDuplicates) {
      this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} for repeating nullifiers present in state trees`);
      return false;
    }

    return true;
  }
}
