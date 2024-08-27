import { EthCheatCodes } from '@aztec/aztec.js';
import { type L2Block, type Signature } from '@aztec/circuit-types';
import { type L1PublishBlockStats, type L1PublishProofStats } from '@aztec/circuit-types/stats';
import {
  AZTEC_SLOT_DURATION,
  ETHEREUM_SLOT_DURATION,
  EthAddress,
  type Header,
  IS_DEV_NET,
  type Proof,
} from '@aztec/circuits.js';
import { createEthereumChain } from '@aztec/ethereum';
import { type Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { serializeToBuffer } from '@aztec/foundation/serialize';
import { InterruptibleSleep } from '@aztec/foundation/sleep';
import { Timer } from '@aztec/foundation/timer';
import { AvailabilityOracleAbi, RollupAbi } from '@aztec/l1-artifacts';
import { type TelemetryClient } from '@aztec/telemetry-client';

import pick from 'lodash.pick';
import {
  type GetContractReturnType,
  type Hex,
  type HttpTransport,
  type PrivateKeyAccount,
  type PublicClient,
  type WalletClient,
  createPublicClient,
  createWalletClient,
  getAddress,
  getContract,
  hexToBytes,
  http,
} from 'viem';
import { privateKeyToAccount } from 'viem/accounts';
import type * as chains from 'viem/chains';

import { type PublisherConfig, type TxSenderConfig } from './config.js';
import { L1PublisherMetrics } from './l1-publisher-metrics.js';

/**
 * Stats for a sent transaction.
 */
export type TransactionStats = {
  /** Hash of the transaction. */
  transactionHash: string;
  /** Size in bytes of the tx calldata */
  calldataSize: number;
  /** Gas required to pay for the calldata inclusion (depends on size and number of zeros)  */
  calldataGas: number;
};

/**
 * Minimal information from a tx receipt.
 */
export type MinimalTransactionReceipt = {
  /** True if the tx was successful, false if reverted. */
  status: boolean;
  /** Hash of the transaction. */
  transactionHash: string;
  /** Effective gas used by the tx. */
  gasUsed: bigint;
  /** Effective gas price paid by the tx. */
  gasPrice: bigint;
  /** Logs emitted in this tx. */
  logs: any[];
};

/** Arguments to the process method of the rollup contract */
export type L1ProcessArgs = {
  /** The L2 block header. */
  header: Buffer;
  /** A root of the archive tree after the L2 block is applied. */
  archive: Buffer;
  /** The L2 block's leaf in the archive tree. */
  blockHash: Buffer;
  /** L2 block body. */
  body: Buffer;
  /** Attestations */
  attestations?: Signature[];
};

/** Arguments to the submitProof method of the rollup contract */
export type L1SubmitProofArgs = {
  /** The L2 block header. */
  header: Buffer;
  /** A root of the archive tree after the L2 block is applied. */
  archive: Buffer;
  /** Identifier of the prover. */
  proverId: Buffer;
  /** The proof for the block. */
  proof: Buffer;
  /** The aggregation object for the block's proof. */
  aggregationObject: Buffer;
};

export type MetadataForSlot = {
  proposer: EthAddress;
  slot: bigint;
  pendingBlockNumber: bigint;
  archive: Buffer;
};

/**
 * Publishes L2 blocks to L1. This implementation does *not* retry a transaction in
 * the event of network congestion, but should work for local development.
 * - If sending (not mining) a tx fails, it retries indefinitely at 1-minute intervals.
 * - If the tx is not mined, keeps polling indefinitely at 1-second intervals.
 *
 * Adapted from https://github.com/AztecProtocol/aztec2-internal/blob/master/falafel/src/rollup_publisher.ts.
 */
export class L1Publisher {
  private interruptibleSleep = new InterruptibleSleep();
  private sleepTimeMs: number;
  private timeTraveler: boolean;
  private interrupted = false;
  private metrics: L1PublisherMetrics;
  private log = createDebugLogger('aztec:sequencer:publisher');

  private availabilityOracleContract: GetContractReturnType<
    typeof AvailabilityOracleAbi,
    WalletClient<HttpTransport, chains.Chain, PrivateKeyAccount>
  >;
  private rollupContract: GetContractReturnType<
    typeof RollupAbi,
    WalletClient<HttpTransport, chains.Chain, PrivateKeyAccount>
  >;
  private publicClient: PublicClient<HttpTransport, chains.Chain>;
  private account: PrivateKeyAccount;

  private ethCheatCodes: EthCheatCodes;

  constructor(config: TxSenderConfig & PublisherConfig, client: TelemetryClient) {
    this.sleepTimeMs = config?.l1PublishRetryIntervalMS ?? 60_000;
    this.timeTraveler = config?.timeTraveler ?? false;
    this.metrics = new L1PublisherMetrics(client, 'L1Publisher');

    const { l1RpcUrl: rpcUrl, l1ChainId: chainId, publisherPrivateKey, l1Contracts } = config;
    const chain = createEthereumChain(rpcUrl, chainId);
    this.account = privateKeyToAccount(publisherPrivateKey);
    const walletClient = createWalletClient({
      account: this.account,
      chain: chain.chainInfo,
      transport: http(chain.rpcUrl),
    });

    this.publicClient = createPublicClient({
      chain: chain.chainInfo,
      transport: http(chain.rpcUrl),
    });

    this.availabilityOracleContract = getContract({
      address: getAddress(l1Contracts.availabilityOracleAddress.toString()),
      abi: AvailabilityOracleAbi,
      client: walletClient,
    });
    this.rollupContract = getContract({
      address: getAddress(l1Contracts.rollupAddress.toString()),
      abi: RollupAbi,
      client: walletClient,
    });

    this.ethCheatCodes = new EthCheatCodes(rpcUrl);
  }

  public async amIAValidator(): Promise<boolean> {
    return await this.rollupContract.read.isValidator([this.account.address]);
  }

  public async getValidatorCount(): Promise<bigint> {
    return BigInt(await this.rollupContract.read.getValidatorCount());
  }

  public getSenderAddress(): Promise<EthAddress> {
    return Promise.resolve(EthAddress.fromString(this.account.address));
  }

  public async willSimulationFail(slot: bigint): Promise<boolean> {
    // @note  When simulating or estimating gas, `viem` will use the CURRENT state of the chain
    //        and not the state in the next block. Meaning that the timestamp will be the same as
    //        the previous block, which means that the slot will also be the same.
    //        This effectively means that if we try to simulate for the first L1 block where we
    //        will be proposer, we will have a failure as the slot have not yet changed.
    // @todo  #8110

    if (IS_DEV_NET) {
      return false;
    }

    const currentSlot = BigInt(await this.rollupContract.read.getCurrentSlot());
    return currentSlot != slot;
  }

  // @note Assumes that all ethereum slots have blocks
  // Using next Ethereum block so we do NOT need to wait for it being mined before seeing the effect
  public async getMetadataForSlotAtNextEthBlock(): Promise<MetadataForSlot> {
    const ts = BigInt((await this.publicClient.getBlock()).timestamp + BigInt(ETHEREUM_SLOT_DURATION));

    const [submitter, slot, pendingBlockCount, archive] = await Promise.all([
      this.rollupContract.read.getProposerAt([ts]),
      this.rollupContract.read.getSlotAt([ts]),
      this.rollupContract.read.pendingBlockCount(),
      this.rollupContract.read.archive(),
    ]);

    return {
      proposer: EthAddress.fromString(submitter),
      slot,
      pendingBlockNumber: pendingBlockCount - 1n,
      archive: Buffer.from(archive.replace('0x', ''), 'hex'),
    };
  }

  public async getCurrentEpochCommittee(): Promise<EthAddress[]> {
    const committee = await this.rollupContract.read.getCurrentEpochCommittee();
    return committee.map(EthAddress.fromString);
  }

  checkIfTxsAreAvailable(block: L2Block): Promise<boolean> {
    const args = [`0x${block.body.getTxsEffectsHash().toString('hex').padStart(64, '0')}`] as const;
    return this.availabilityOracleContract.read.isAvailable(args);
  }

  async getTransactionStats(txHash: string): Promise<TransactionStats | undefined> {
    const tx = await this.publicClient.getTransaction({ hash: txHash as Hex });
    if (!tx) {
      return undefined;
    }
    const calldata = hexToBytes(tx.input);
    return {
      transactionHash: tx.hash,
      calldataSize: calldata.length,
      calldataGas: getCalldataGasUsage(calldata),
    };
  }

  /**
   * Publishes L2 block on L1.
   * @param block - L2 block to publish.
   * @returns True once the tx has been confirmed and is successful, false on revert or interrupt, blocks otherwise.
   */
  public async processL2Block(block: L2Block, attestations?: Signature[]): Promise<boolean> {
    const ctx = {
      blockNumber: block.number,
      slotNumber: block.header.globalVariables.slotNumber.toBigInt(),
      blockHash: block.hash().toString(),
    };

    if (await this.willSimulationFail(block.header.globalVariables.slotNumber.toBigInt())) {
      // @note  See comment in willSimulationFail for more information
      this.log.info(`Simulation will fail for slot ${block.header.globalVariables.slotNumber.toBigInt()}`);
      return false;
    }

    const processTxArgs = {
      header: block.header.toBuffer(),
      archive: block.archive.root.toBuffer(),
      blockHash: block.header.hash().toBuffer(),
      body: block.body.toBuffer(),
      attestations,
    };

    // Process block and publish the body if needed (if not already published)
    if (!this.interrupted) {
      let txHash;
      const timer = new Timer();

      if (await this.checkIfTxsAreAvailable(block)) {
        this.log.verbose(`Transaction effects of block ${block.number} already published.`, ctx);
        txHash = await this.sendProcessTx(processTxArgs);
      } else {
        txHash = await this.sendPublishAndProcessTx(processTxArgs);
      }

      if (!txHash) {
        this.log.info(`Failed to publish block ${block.number} to L1`, ctx);
        return false;
      }

      const receipt = await this.getTransactionReceipt(txHash);
      if (!receipt) {
        this.log.info(`Failed to get receipt for tx ${txHash}`, ctx);
        return false;
      }

      // Tx was mined successfully
      if (receipt.status) {
        const tx = await this.getTransactionStats(txHash);
        const stats: L1PublishBlockStats = {
          ...pick(receipt, 'gasPrice', 'gasUsed', 'transactionHash'),
          ...pick(tx!, 'calldataGas', 'calldataSize'),
          ...block.getStats(),
          eventName: 'rollup-published-to-l1',
        };
        this.log.info(`Published L2 block to L1 rollup contract`, { ...stats, ...ctx });
        this.metrics.recordProcessBlockTx(timer.ms(), stats);

        await this.commitTimeJump(block.header.globalVariables.slotNumber.toBigInt() + 1n);

        return true;
      }

      this.metrics.recordFailedTx('process');

      this.log.error(`Rollup.process tx status failed: ${receipt.transactionHash}`, ctx);
      await this.sleepOrInterrupted();
    }

    this.log.verbose('L2 block data syncing interrupted while processing blocks.', ctx);
    return false;
  }

  async commitTimeJump(slot: bigint) {
    // @note  So, we are cheating a bit here. Since the tests are running anvil auto-mine
    //        no blocks are coming around unless we do something, and since we cannot push
    //        more blocks within the same slot (when `IS_DEV_NET = false`), we can just
    //        fast forward the anvil chain such that the next block will be the one we need.
    //        this means that we are forwarding to time of the next slot - 12 seconds such that
    //        the NEXT ethereum block will be within the new slot.
    //        If the slot duration of L2 is just 1 L1 slot, then this will not do anything, as
    //        the time to jump to is current time.
    //
    //        Time jumps only allowed into the future.
    //
    //        If this is run on top of a real chain, the time lords will catch you.
    //
    // @todo  #8153

    if (!this.timeTraveler) {
      return;
    }

    // If the aztec slot duration is same length as the ethereum slot duration, we don't need to do anything
    if ((ETHEREUM_SLOT_DURATION as number) === (AZTEC_SLOT_DURATION as number)) {
      return;
    }

    const [currentTime, timeStampForSlot] = await Promise.all([
      this.ethCheatCodes.timestamp(),
      this.rollupContract.read.getTimestampForSlot([slot]),
    ]);

    // @note  We progress the time to the next slot AND mine the block.
    //        This means that the next effective block will be ETHEREUM_SLOT_DURATION after that.
    //        This will cause issues if slot duration is equal to one (1) L1 slot, and sequencer selection is run
    //        The reason is that simulations on ANVIL cannot be run with timestamp + x, so we need to "BE" there.
    // @todo  #8110
    const timestamp = timeStampForSlot; //  - BigInt(ETHEREUM_SLOT_DURATION);

    if (timestamp > currentTime) {
      this.log.info(`Committing time jump to slot ${slot}`);
      await this.ethCheatCodes.warp(Number(timestamp));
    }
  }

  public async submitProof(
    header: Header,
    archiveRoot: Fr,
    proverId: Fr,
    aggregationObject: Fr[],
    proof: Proof,
  ): Promise<boolean> {
    const ctx = { blockNumber: header.globalVariables.blockNumber, slotNumber: header.globalVariables.slotNumber };

    const txArgs: L1SubmitProofArgs = {
      header: header.toBuffer(),
      archive: archiveRoot.toBuffer(),
      proverId: proverId.toBuffer(),
      aggregationObject: serializeToBuffer(aggregationObject),
      proof: proof.withoutPublicInputs(),
    };

    // Process block
    if (!this.interrupted) {
      const timer = new Timer();
      const txHash = await this.sendSubmitProofTx(txArgs);
      if (!txHash) {
        return false;
      }

      const receipt = await this.getTransactionReceipt(txHash);
      if (!receipt) {
        return false;
      }

      // Tx was mined successfully
      if (receipt.status) {
        const tx = await this.getTransactionStats(txHash);
        const stats: L1PublishProofStats = {
          ...pick(receipt, 'gasPrice', 'gasUsed', 'transactionHash'),
          ...pick(tx!, 'calldataGas', 'calldataSize'),
          eventName: 'proof-published-to-l1',
        };
        this.log.info(`Published proof to L1 rollup contract`, { ...stats, ...ctx });
        this.metrics.recordSubmitProof(timer.ms(), stats);
        return true;
      }

      this.metrics.recordFailedTx('submitProof');
      this.log.error(`Rollup.submitProof tx status failed: ${receipt.transactionHash}`, ctx);
      await this.sleepOrInterrupted();
    }

    this.log.verbose('L2 block data syncing interrupted while processing blocks.', ctx);
    return false;
  }

  /**
   * Calling `interrupt` will cause any in progress call to `publishRollup` to return `false` asap.
   * Be warned, the call may return false even if the tx subsequently gets successfully mined.
   * In practice this shouldn't matter, as we'll only ever be calling `interrupt` when we know it's going to fail.
   * A call to `restart` is required before you can continue publishing.
   */
  public interrupt() {
    this.interrupted = true;
    this.interruptibleSleep.interrupt();
  }

  /** Restarts the publisher after calling `interrupt`. */
  public restart() {
    this.interrupted = false;
  }

  private async sendSubmitProofTx(submitProofArgs: L1SubmitProofArgs): Promise<string | undefined> {
    try {
      const size = Object.values(submitProofArgs).reduce((acc, arg) => acc + arg.length, 0);
      this.log.info(`SubmitProof size=${size} bytes`);

      const { header, archive, proverId, aggregationObject, proof } = submitProofArgs;
      const args = [
        `0x${header.toString('hex')}`,
        `0x${archive.toString('hex')}`,
        `0x${proverId.toString('hex')}`,
        `0x${aggregationObject.toString('hex')}`,
        `0x${proof.toString('hex')}`,
      ] as const;

      await this.rollupContract.simulate.submitBlockRootProof(args, {
        account: this.account,
      });

      return await this.rollupContract.write.submitBlockRootProof(args, {
        account: this.account,
      });
    } catch (err) {
      this.log.error(`Rollup submit proof failed`, err);
      return undefined;
    }
  }

  private async sendPublishTx(encodedBody: Buffer): Promise<string | undefined> {
    if (!this.interrupted) {
      try {
        this.log.info(`TxEffects size=${encodedBody.length} bytes`);
        const args = [`0x${encodedBody.toString('hex')}`] as const;

        await this.availabilityOracleContract.simulate.publish(args, {
          account: this.account,
        });

        return await this.availabilityOracleContract.write.publish(args, {
          account: this.account,
        });
      } catch (err) {
        this.log.error(`TxEffects publish failed`, err);
        return undefined;
      }
    }
  }

  private async sendProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined> {
    if (!this.interrupted) {
      try {
        if (encodedData.attestations) {
          const attestations = encodedData.attestations.map(attest => attest.toViemSignature());
          const args = [
            `0x${encodedData.header.toString('hex')}`,
            `0x${encodedData.archive.toString('hex')}`,
            `0x${encodedData.blockHash.toString('hex')}`,
            attestations,
          ] as const;

          await this.rollupContract.simulate.process(args, { account: this.account });

          return await this.rollupContract.write.process(args, {
            account: this.account,
          });
        } else {
          const args = [
            `0x${encodedData.header.toString('hex')}`,
            `0x${encodedData.archive.toString('hex')}`,
            `0x${encodedData.blockHash.toString('hex')}`,
          ] as const;

          await this.rollupContract.simulate.process(args, { account: this.account });

          return await this.rollupContract.write.process(args, {
            account: this.account,
          });
        }
      } catch (err) {
        this.log.error(`Rollup publish failed`, err);
        return undefined;
      }
    }
  }

  private async sendPublishAndProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined> {
    if (!this.interrupted) {
      try {
        if (encodedData.attestations) {
          const attestations = encodedData.attestations.map(attest => attest.toViemSignature());
          const args = [
            `0x${encodedData.header.toString('hex')}`,
            `0x${encodedData.archive.toString('hex')}`,
            `0x${encodedData.blockHash.toString('hex')}`,
            attestations,
            `0x${encodedData.body.toString('hex')}`,
          ] as const;

          // Using simulate to get a meaningful error message
          await this.rollupContract.simulate.publishAndProcess(args, {
            account: this.account,
          });

          return await this.rollupContract.write.publishAndProcess(args, {
            account: this.account,
          });
        } else {
          const args = [
            `0x${encodedData.header.toString('hex')}`,
            `0x${encodedData.archive.toString('hex')}`,
            `0x${encodedData.blockHash.toString('hex')}`,
            `0x${encodedData.body.toString('hex')}`,
          ] as const;

          await this.rollupContract.simulate.publishAndProcess(args, {
            account: this.account,
          });

          return await this.rollupContract.write.publishAndProcess(args, {
            account: this.account,
          });
        }
      } catch (err) {
        this.log.error(`Rollup publish failed`, err);
        return undefined;
      }
    }
  }

  /**
   * Returns a tx receipt if the tx has been mined.
   * @param txHash - Hash of the tx to look for.
   * @returns Undefined if the tx hasn't been mined yet, the receipt otherwise.
   */
  async getTransactionReceipt(txHash: string): Promise<MinimalTransactionReceipt | undefined> {
    while (!this.interrupted) {
      try {
        const receipt = await this.publicClient.getTransactionReceipt({
          hash: txHash as Hex,
        });

        if (receipt) {
          if (receipt.transactionHash !== txHash) {
            throw new Error(`Tx hash mismatch: ${receipt.transactionHash} !== ${txHash}`);
          }

          return {
            status: receipt.status === 'success',
            transactionHash: txHash,
            gasUsed: receipt.gasUsed,
            gasPrice: receipt.effectiveGasPrice,
            logs: receipt.logs,
          };
        }

        this.log.debug(`Receipt not found for tx hash ${txHash}`);
        return undefined;
      } catch (err) {
        //this.log.error(`Error getting tx receipt`, err);
        await this.sleepOrInterrupted();
      }
    }
  }

  protected async sleepOrInterrupted() {
    await this.interruptibleSleep.sleep(this.sleepTimeMs);
  }
}

/**
 * Returns cost of calldata usage in Ethereum.
 * @param data - Calldata.
 * @returns 4 for each zero byte, 16 for each nonzero.
 */
function getCalldataGasUsage(data: Uint8Array) {
  return data.filter(byte => byte === 0).length * 4 + data.filter(byte => byte !== 0).length * 16;
}
