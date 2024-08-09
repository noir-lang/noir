import {
  type AztecAddress,
  ETHEREUM_SLOT_DURATION,
  type EthAddress,
  GasFees,
  GlobalVariables,
} from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

/**
 * Reads values from L1 state that is used for the global values.
 */
export interface L1GlobalReader {
  /**
   * Fetches the version of the rollup contract.
   * @returns The version of the rollup contract.
   */
  getVersion(): Promise<bigint>;
  /**
   * Gets the chain id.
   * @returns The chain id.
   */
  getChainId(): Promise<bigint>;

  /**
   * Gets the current L1 time.
   * @returns The current L1 time.
   */
  getL1CurrentTime(): Promise<bigint>;

  /**
   * Gets the current slot.
   * @returns The current slot.
   */
  getCurrentSlot(): Promise<bigint>;

  /**
   * Get the slot for a specific timestamp.
   * @param timestamp - The timestamp to get the slot for.
   */
  getSlotAt(timestamp: readonly [bigint]): Promise<bigint>;

  /**
   * Gets the timestamp for a slot
   * @param slot - The slot to get the timestamp for.
   * @returns The timestamp for the slot.
   */
  getTimestampForSlot(slot: readonly [bigint]): Promise<bigint>;
}

/**
 * Builds global variables from L1 state.
 */
export interface GlobalVariableBuilder {
  /**
   * Builds global variables.
   * @param blockNumber - The block number to build global variables for.
   * @param coinbase - The address to receive block reward.
   * @param feeRecipient - The address to receive fees.
   * @returns The global variables for the given block number.
   */
  buildGlobalVariables(blockNumber: Fr, coinbase: EthAddress, feeRecipient: AztecAddress): Promise<GlobalVariables>;
}

/**
 * Simple test implementation of a builder that uses the minimum time possible for the global variables.
 * Also uses a "hack" to make use of the warp cheatcode that manipulates time on Aztec.
 */
export class SimpleTestGlobalVariableBuilder implements GlobalVariableBuilder {
  private log = createDebugLogger('aztec:sequencer:simple_test_global_variable_builder');
  constructor(private readonly reader: L1GlobalReader) {}

  /**
   * Simple builder of global variables that use the minimum time possible.
   * @param blockNumber - The block number to build global variables for.
   * @param coinbase - The address to receive block reward.
   * @param feeRecipient - The address to receive fees.
   * @returns The global variables for the given block number.
   */
  public async buildGlobalVariables(
    blockNumber: Fr,
    coinbase: EthAddress,
    feeRecipient: AztecAddress,
  ): Promise<GlobalVariables> {
    // Not just the current slot, the slot of the next block.
    const ts = (await this.reader.getL1CurrentTime()) + BigInt(ETHEREUM_SLOT_DURATION);

    const slot = await this.reader.getSlotAt([ts]);
    const timestamp = await this.reader.getTimestampForSlot([slot]);

    const slotFr = new Fr(slot);
    const timestampFr = new Fr(timestamp);

    const version = new Fr(await this.reader.getVersion());
    const chainId = new Fr(await this.reader.getChainId());

    const gasFees = GasFees.default();
    const globalVariables = new GlobalVariables(
      chainId,
      version,
      blockNumber,
      slotFr,
      timestampFr,
      coinbase,
      feeRecipient,
      gasFees,
    );
    this.log.debug(`Built global variables for block ${blockNumber}`, globalVariables.toJSON());
    return globalVariables;
  }
}
