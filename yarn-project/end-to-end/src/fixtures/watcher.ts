import { type DebugLogger, type EthCheatCodes, createDebugLogger } from '@aztec/aztec.js';
import { type EthAddress } from '@aztec/circuits.js';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { RollupAbi } from '@aztec/l1-artifacts';

import { type GetContractReturnType, type HttpTransport, type PublicClient, getAddress, getContract } from 'viem';
import type * as chains from 'viem/chains';

export class Watcher {
  private rollup: GetContractReturnType<typeof RollupAbi, PublicClient<HttpTransport, chains.Chain>>;

  private filledRunningPromise?: RunningPromise;

  private logger: DebugLogger = createDebugLogger(`aztec:utils:watcher`);

  constructor(
    private cheatcodes: EthCheatCodes,
    rollupAddress: EthAddress,
    publicClient: PublicClient<HttpTransport, chains.Chain>,
  ) {
    this.rollup = getContract({
      address: getAddress(rollupAddress.toString()),
      abi: RollupAbi,
      client: publicClient,
    });

    this.logger.info(`Watcher created for rollup at ${rollupAddress}`);
  }

  start() {
    if (this.filledRunningPromise) {
      throw new Error('Watcher already watching for filled slot');
    }
    this.filledRunningPromise = new RunningPromise(() => this.mineIfSlotFilled(), 1000);
    this.filledRunningPromise.start();
    this.logger.info(`Watcher started`);
  }

  async stop() {
    await this.filledRunningPromise?.stop();
  }

  async mineIfSlotFilled() {
    try {
      const currentSlot = await this.rollup.read.getCurrentSlot();
      const pendingBlockNumber = BigInt(await this.rollup.read.pendingBlockCount()) - 1n;
      const [, , lastSlotNumber] = await this.rollup.read.blocks([pendingBlockNumber]);

      if (currentSlot === lastSlotNumber) {
        // We should jump to the next slot
        const timestamp = await this.rollup.read.getTimestampForSlot([currentSlot + 1n]);
        try {
          await this.cheatcodes.warp(Number(timestamp));
        } catch (e) {
          this.logger.error(`Failed to warp to timestamp ${timestamp}: ${e}`);
        }

        this.logger.info(`Slot ${currentSlot} was filled, jumped to next slot`);
      }
    } catch (err) {
      this.logger.error('mineIfSlotFilled failed');
    }
  }
}
