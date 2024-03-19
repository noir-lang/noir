import { Tx } from '@aztec/circuit-types';
import { Fr, GlobalVariables } from '@aztec/circuits.js';
import { Logger, createDebugLogger } from '@aztec/foundation/log';

import { ProcessedTx } from './processed_tx.js';

export interface NullifierSource {
  getNullifierIndex: (nullifier: Fr) => Promise<bigint | undefined>;
}

// prefer symbols over booleans so it's clear what the intention is
// vs returning true/false is tied to the function name
// eg. isDoubleSpend vs isValidChain assign different meanings to booleans
const VALID_TX = Symbol('valid_tx');
const INVALID_TX = Symbol('invalid_tx');

type TxValidationStatus = typeof VALID_TX | typeof INVALID_TX;

export class TxValidator {
  #log: Logger;
  #globalVariables: GlobalVariables;
  #nullifierSource: NullifierSource;

  constructor(
    nullifierSource: NullifierSource,
    globalVariables: GlobalVariables,
    log = createDebugLogger('aztec:sequencer:tx_validator'),
  ) {
    this.#nullifierSource = nullifierSource;
    this.#globalVariables = globalVariables;

    this.#log = log;
  }

  /**
   * Validates a list of transactions.
   * @param txs - The transactions to validate.
   * @returns A tuple of valid and invalid transactions.
   */
  public async validateTxs<T extends Tx | ProcessedTx>(txs: T[]): Promise<[validTxs: T[], invalidTxs: T[]]> {
    const validTxs: T[] = [];
    const invalidTxs: T[] = [];
    const thisBlockNullifiers = new Set<bigint>();

    for (const tx of txs) {
      if (this.#validateMetadata(tx) === INVALID_TX) {
        invalidTxs.push(tx);
        continue;
      }

      if ((await this.#validateNullifiers(tx, thisBlockNullifiers)) === INVALID_TX) {
        invalidTxs.push(tx);
        continue;
      }

      validTxs.push(tx);
    }

    return [validTxs, invalidTxs];
  }

  /**
   * It rejects transactions with the wrong chain id.
   * @param tx - The transaction.
   * @returns Whether the transaction is valid.
   */
  #validateMetadata(tx: Tx | ProcessedTx): TxValidationStatus {
    if (!tx.data.constants.txContext.chainId.equals(this.#globalVariables.chainId)) {
      this.#log.warn(
        `Rejecting tx ${Tx.getHash(
          tx,
        )} because of incorrect chain ${tx.data.constants.txContext.chainId.toString()} != ${this.#globalVariables.chainId.toString()}`,
      );
      return INVALID_TX;
    }

    return VALID_TX;
  }

  /**
   * It looks for duplicate nullifiers:
   * - in the same transaction
   * - in the same block
   * - in the nullifier tree
   *
   * Nullifiers prevent double spends in a private context.
   *
   * @param tx - The transaction.
   * @returns Whether this is a problematic double spend that the L1 contract would reject.
   */
  async #validateNullifiers(tx: Tx | ProcessedTx, thisBlockNullifiers: Set<bigint>): Promise<TxValidationStatus> {
    const newNullifiers = TxValidator.#extractNullifiers(tx);

    // Ditch this tx if it has a repeated nullifiers
    const uniqueNullifiers = new Set(newNullifiers);
    if (uniqueNullifiers.size !== newNullifiers.length) {
      this.#log.warn(`Rejecting tx for emitting duplicate nullifiers, tx hash ${Tx.getHash(tx)}`);
      return INVALID_TX;
    }

    for (const nullifier of newNullifiers) {
      if (thisBlockNullifiers.has(nullifier)) {
        this.#log.warn(`Rejecting tx for repeating a in the same block, tx hash ${Tx.getHash(tx)}`);
        return INVALID_TX;
      }

      thisBlockNullifiers.add(nullifier);
    }

    const nullifierIndexes = await Promise.all(
      newNullifiers.map(n => this.#nullifierSource.getNullifierIndex(new Fr(n))),
    );

    const hasDuplicates = nullifierIndexes.some(index => index !== undefined);
    if (hasDuplicates) {
      this.#log.warn(`Rejecting tx for repeating nullifiers from the past, tx hash ${Tx.getHash(tx)}`);
      return INVALID_TX;
    }

    return VALID_TX;
  }

  static #extractNullifiers(tx: Tx | ProcessedTx): bigint[] {
    return [...tx.data.endNonRevertibleData.newNullifiers, ...tx.data.end.newNullifiers]
      .filter(x => !x.isEmpty())
      .map(x => x.value.toBigInt());
  }
}
