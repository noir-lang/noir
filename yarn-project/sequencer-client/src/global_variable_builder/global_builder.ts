import { AztecAddress, EthAddress, GlobalVariables } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

/**
 * Reads values from L1 state that is used for the global values.
 */
export interface L1GlobalReader {
  /**
   * Fetches the last timestamp that a block was processed by the contract.
   * @returns The last timestamp that a block was processed by the contract.
   */
  getLastTimestamp(): Promise<bigint>;
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
   * Gets the last time L2 was warped as tracked by the rollup contract.
   * @returns The warped time.
   */
  getLastWarpedBlockTs(): Promise<bigint>;
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
    let lastTimestamp = new Fr(await this.reader.getLastTimestamp());
    const version = new Fr(await this.reader.getVersion());
    const chainId = new Fr(await this.reader.getChainId());

    // TODO(rahul) - fix #1614. By using the cheatcode warp to modify L2 time,
    // txs in the next rollup would have same time as the txs in the current rollup (i.e. the rollup that was warped).
    // So, for now you check if L2 time was warped and if so, serve warpedTime + 1 to txs in the new rollup.
    // Check if L2 time was warped in the last rollup by checking if current L1 time is same as the warpedTime (stored on the rollup contract).
    // more details at https://github.com/AztecProtocol/aztec-packages/issues/1614

    const currTimestamp = await this.reader.getL1CurrentTime();
    const rollupWarpTime = await this.reader.getLastWarpedBlockTs();
    const isLastBlockWarped = rollupWarpTime === currTimestamp;
    if (isLastBlockWarped) {
      lastTimestamp = new Fr(lastTimestamp.value + 1n);
    }

    this.log(
      `Built global variables for block ${blockNumber}: (${chainId}, ${version}, ${blockNumber}, ${lastTimestamp}, ${coinbase}, ${feeRecipient})`,
    );

    return new GlobalVariables(chainId, version, blockNumber, lastTimestamp, coinbase, feeRecipient);
  }
}
