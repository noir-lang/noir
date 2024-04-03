import { type ProcessedTx, Tx } from '@aztec/circuit-types';
import {
  type AztecAddress,
  type EthAddress,
  Fr,
  type GlobalVariables,
  type PublicCallRequest,
} from '@aztec/circuits.js';
import { pedersenHash } from '@aztec/foundation/crypto';
import { type Logger, createDebugLogger } from '@aztec/foundation/log';
import { getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';
import { type ContractDataSource } from '@aztec/types/contracts';

import { AbstractPhaseManager, PublicKernelPhase } from './abstract_phase_manager.js';

/** A source of what nullifiers have been committed to the state trees */
export interface NullifierSource {
  getNullifierIndex: (nullifier: Fr) => Promise<bigint | undefined>;
}

/** Provides a view into public contract state */
export interface PublicStateSource {
  storageRead: (contractAddress: AztecAddress, slot: Fr) => Promise<Fr>;
}

// prefer symbols over booleans so it's clear what the intention is
// vs returning true/false is tied to the function name
// eg. isDoubleSpend vs isValidChain assign different meanings to booleans
const VALID_TX = Symbol('valid_tx');
const INVALID_TX = Symbol('invalid_tx');

type TxValidationStatus = typeof VALID_TX | typeof INVALID_TX;

// the storage slot associated with "storage.balances"
const GAS_TOKEN_BALANCES_SLOT = new Fr(1);

type FeeValidationConfig = {
  gasPortalAddress: EthAddress;
  allowedFeePaymentContractClasses: Fr[];
  allowedFeePaymentContractInstances: AztecAddress[];
};

export class TxValidator {
  #log: Logger;
  #globalVariables: GlobalVariables;
  #nullifierSource: NullifierSource;
  #publicStateSource: PublicStateSource;
  #contractDataSource: ContractDataSource;
  #feeValidationConfig: FeeValidationConfig;

  constructor(
    nullifierSource: NullifierSource,
    publicStateSource: PublicStateSource,
    contractDataSource: ContractDataSource,
    globalVariables: GlobalVariables,
    feeValidationConfig: FeeValidationConfig,
    log = createDebugLogger('aztec:sequencer:tx_validator'),
  ) {
    this.#nullifierSource = nullifierSource;
    this.#publicStateSource = publicStateSource;
    this.#contractDataSource = contractDataSource;
    this.#globalVariables = globalVariables;
    this.#feeValidationConfig = feeValidationConfig;
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

      // skip already processed transactions
      if (tx instanceof Tx) {
        if ((await this.#validateFee(tx)) === INVALID_TX) {
          invalidTxs.push(tx);
          continue;
        }
        if ((await this.#validateGasBalance(tx)) === INVALID_TX) {
          invalidTxs.push(tx);
          continue;
        }
      }

      if (this.#validateMaxBlockNumber(tx) === INVALID_TX) {
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
    const newNullifiers = tx.data.getNonEmptyNullifiers().map(x => x.value.toBigInt());

    // Ditch this tx if it has repeated nullifiers
    const uniqueNullifiers = new Set(newNullifiers);
    if (uniqueNullifiers.size !== newNullifiers.length) {
      this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} for emitting duplicate nullifiers`);
      return INVALID_TX;
    }

    for (const nullifier of newNullifiers) {
      if (thisBlockNullifiers.has(nullifier)) {
        this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} for repeating a nullifier in the same block`);
        return INVALID_TX;
      }

      thisBlockNullifiers.add(nullifier);
    }

    const nullifierIndexes = await Promise.all(
      newNullifiers.map(n => this.#nullifierSource.getNullifierIndex(new Fr(n))),
    );

    const hasDuplicates = nullifierIndexes.some(index => index !== undefined);
    if (hasDuplicates) {
      this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} for repeating nullifiers present in state trees`);
      return INVALID_TX;
    }

    return VALID_TX;
  }

  async #validateGasBalance(tx: Tx): Promise<TxValidationStatus> {
    if (!tx.data.forPublic || !tx.data.forPublic.needsTeardown) {
      return VALID_TX;
    }

    const teardownFn = TxValidator.#extractFeeExecutionCall(tx)!;

    // TODO(#1204) if a generator index is used for the derived storage slot of a map, update it here as well
    const slot = pedersenHash([GAS_TOKEN_BALANCES_SLOT, teardownFn.callContext.msgSender]);
    const gasBalance = await this.#publicStateSource.storageRead(
      getCanonicalGasTokenAddress(this.#feeValidationConfig.gasPortalAddress),
      slot,
    );

    // TODO(#5004) calculate fee needed based on tx limits and gas prices
    const gasAmountNeeded = new Fr(1);
    if (gasBalance.lt(gasAmountNeeded)) {
      this.#log.warn(
        `Rejecting tx ${Tx.getHash(
          tx,
        )} because it should pay for gas but has insufficient balance ${gasBalance.toShortString()} < ${gasAmountNeeded.toShortString()}`,
      );
      return INVALID_TX;
    }

    return VALID_TX;
  }

  #validateMaxBlockNumber(tx: Tx | ProcessedTx): TxValidationStatus {
    const target =
      tx instanceof Tx
        ? tx.data.forRollup?.rollupValidationRequests || tx.data.forPublic!.validationRequests.forRollup
        : tx.data.rollupValidationRequests;
    const maxBlockNumber = target.maxBlockNumber;

    if (maxBlockNumber.isSome && maxBlockNumber.value < this.#globalVariables.blockNumber) {
      this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} for low max block number`);
      return INVALID_TX;
    } else {
      return VALID_TX;
    }
  }

  async #validateFee(tx: Tx): Promise<TxValidationStatus> {
    if (!tx.data.forPublic || !tx.data.forPublic.needsTeardown) {
      // TODO check if fees are mandatory and reject this tx
      this.#log.debug(`Tx ${Tx.getHash(tx)} doesn't pay for gas`);
      return VALID_TX;
    }

    const teardownFn = TxValidator.#extractFeeExecutionCall(tx);
    if (!teardownFn) {
      this.#log.warn(
        `Rejecting tx ${Tx.getHash(tx)} because it should pay for gas but has no enqueued teardown function call`,
      );
      return INVALID_TX;
    }

    const fpcAddress = teardownFn.contractAddress;
    const contractClass = await this.#contractDataSource.getContract(fpcAddress);

    if (!contractClass) {
      return INVALID_TX;
    }

    if (fpcAddress.equals(getCanonicalGasTokenAddress(this.#feeValidationConfig.gasPortalAddress))) {
      return VALID_TX;
    }

    for (const allowedContract of this.#feeValidationConfig.allowedFeePaymentContractInstances) {
      if (fpcAddress.equals(allowedContract)) {
        return VALID_TX;
      }
    }

    for (const allowedContractClass of this.#feeValidationConfig.allowedFeePaymentContractClasses) {
      if (contractClass.contractClassId.equals(allowedContractClass)) {
        return VALID_TX;
      }
    }

    return INVALID_TX;
  }

  static #extractFeeExecutionCall(tx: Tx): PublicCallRequest | undefined {
    const {
      // TODO what if there's more than one function call?
      // if we're to enshrine that teardown = 1 function call, then we should turn this into a single function call
      [PublicKernelPhase.TEARDOWN]: [teardownFn],
    } = AbstractPhaseManager.extractEnqueuedPublicCallsByPhase(tx.data, tx.enqueuedPublicFunctionCalls);

    return teardownFn;
  }
}
