import { Tx } from '@aztec/circuit-types';
import { type AztecAddress, Fr } from '@aztec/circuits.js';
import { pedersenHash } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';
import { GasTokenContract } from '@aztec/noir-contracts.js';

import { AbstractPhaseManager, PublicKernelPhase } from '../sequencer/abstract_phase_manager.js';
import { type TxValidator } from './tx_validator.js';

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

  async #validateTxFee(tx: Tx): Promise<boolean> {
    const { [PublicKernelPhase.TEARDOWN]: teardownFns } = AbstractPhaseManager.extractEnqueuedPublicCallsByPhase(
      tx.data,
      tx.enqueuedPublicFunctionCalls,
    );

    if (teardownFns.length === 0) {
      if (this.#requireFees) {
        this.#log.warn(
          `Rejecting tx ${Tx.getHash(tx)} because it should pay for gas but has no enqueued teardown functions`,
        );
        return false;
      } else {
        this.#log.debug(`Tx ${Tx.getHash(tx)} does not pay fees. Skipping balance check.`);
        return true;
      }
    }

    if (teardownFns.length > 1) {
      this.#log.warn(`Rejecting tx ${Tx.getHash(tx)} because it has multiple teardown functions`);
      return false;
    }

    // check that the caller of the teardown function has enough balance to pay for tx costs
    const teardownFn = teardownFns[0];
    const slot = pedersenHash([GasTokenContract.storage.balances.slot, teardownFn.callContext.msgSender]);
    const gasBalance = await this.#publicDataSource.storageRead(this.#gasTokenAddress, slot);

    // TODO(#5004) calculate fee needed based on tx limits and gas prices
    const gasAmountNeeded = new Fr(1);
    if (gasBalance.lt(gasAmountNeeded)) {
      this.#log.warn(
        `Rejecting tx ${Tx.getHash(
          tx,
        )} because it should pay for gas but has insufficient balance ${gasBalance.toShortString()} < ${gasAmountNeeded.toShortString()}`,
      );
      return false;
    }

    return true;
  }
}
