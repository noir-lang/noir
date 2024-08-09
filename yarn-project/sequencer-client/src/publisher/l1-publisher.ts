import { type L2Block } from '@aztec/circuit-types';
import { type L1PublishBlockStats, type L1PublishProofStats } from '@aztec/circuit-types/stats';
import { type EthAddress, type Header, type Proof } from '@aztec/circuits.js';
import { type Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { serializeToBuffer } from '@aztec/foundation/serialize';
import { InterruptibleSleep } from '@aztec/foundation/sleep';
import { Timer } from '@aztec/foundation/timer';
import { type TelemetryClient } from '@aztec/telemetry-client';

import pick from 'lodash.pick';

import { type L2BlockReceiver } from '../receiver.js';
import { type PublisherConfig } from './config.js';
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
 * Minimal information from a tx receipt returned by an L1PublisherTxSender.
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

/**
 * @notice An attestation for the sequencing model.
 * @todo    This is not where it belongs. But I think we should do a bigger rewrite of some of
 *          this spaghetti.
 */
export type Attestation = { isEmpty: boolean; v: number; r: `0x${string}`; s: `0x${string}` };

/**
 * Pushes txs to the L1 chain and waits for their completion.
 */
export interface L1PublisherTxSender {
  /** Attests to the given archive root. */
  attest(archive: `0x${string}`): Promise<Attestation>;

  /** Returns the EOA used for sending txs to L1.  */
  getSenderAddress(): Promise<EthAddress>;

  /** Returns the address of the L2 proposer at the NEXT Ethereum block zero if anyone can submit. */
  getProposerAtNextEthBlock(): Promise<EthAddress>;

  /**
   * Publishes tx effects to Availability Oracle.
   * @param encodedBody - Encoded block body.
   * @returns The hash of the mined tx.
   */
  sendPublishTx(encodedBody: Buffer): Promise<string | undefined>;

  /**
   * Sends a tx to the L1 rollup contract with a new L2 block. Returns once the tx has been mined.
   * @param encodedData - Serialized data for processing the new L2 block.
   * @returns The hash of the mined tx.
   */
  sendProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined>;

  /**
   * Publishes tx effects to availability oracle and send L2 block to rollup contract
   * @param encodedData - Data for processing the new L2 block.
   * @returns The hash of the tx.
   */
  sendPublishAndProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined>;

  /**
   * Sends a tx to the L1 rollup contract with a proof. Returns once the tx has been mined.
   * @param encodedData - Serialized data for processing the new L2 block.
   * @returns The hash of the mined tx.
   */
  sendSubmitProofTx(submitProofArgs: L1SubmitProofArgs): Promise<string | undefined>;

  /**
   * Returns a tx receipt if the tx has been mined.
   * @param txHash - Hash of the tx to look for.
   * @returns Undefined if the tx hasn't been mined yet, the receipt otherwise.
   */
  getTransactionReceipt(txHash: string): Promise<MinimalTransactionReceipt | undefined>;

  /**
   * Returns info on a tx by calling eth_getTransaction.
   * @param txHash - Hash of the tx to look for.
   */
  getTransactionStats(txHash: string): Promise<TransactionStats | undefined>;

  /**
   * Returns the current archive root.
   * @returns The current archive root of the rollup contract.
   */
  getCurrentArchive(): Promise<Buffer>;

  /**
   * Checks if the transaction effects of the given block are available.
   * @param block - The block of which to check whether txs are available.
   * @returns True if the txs are available, false otherwise.
   */
  checkIfTxsAreAvailable(block: L2Block): Promise<boolean>;
}

/** Arguments to the process method of the rollup contract */
export type L1ProcessArgs = {
  /** The L2 block header. */
  header: Buffer;
  /** A root of the archive tree after the L2 block is applied. */
  archive: Buffer;
  /** L2 block body. */
  body: Buffer;
  /** Attestations */
  attestations?: Attestation[];
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

/**
 * Publishes L2 blocks to L1. This implementation does *not* retry a transaction in
 * the event of network congestion, but should work for local development.
 * - If sending (not mining) a tx fails, it retries indefinitely at 1-minute intervals.
 * - If the tx is not mined, keeps polling indefinitely at 1-second intervals.
 *
 * Adapted from https://github.com/AztecProtocol/aztec2-internal/blob/master/falafel/src/rollup_publisher.ts.
 */
export class L1Publisher implements L2BlockReceiver {
  private interruptibleSleep = new InterruptibleSleep();
  private sleepTimeMs: number;
  private interrupted = false;
  private metrics: L1PublisherMetrics;
  private log = createDebugLogger('aztec:sequencer:publisher');

  constructor(private txSender: L1PublisherTxSender, client: TelemetryClient, config?: PublisherConfig) {
    this.sleepTimeMs = config?.l1PublishRetryIntervalMS ?? 60_000;
    this.metrics = new L1PublisherMetrics(client, 'L1Publisher');
  }

  public async attest(archive: `0x${string}`): Promise<Attestation> {
    return await this.txSender.attest(archive);
  }

  public async senderAddress(): Promise<EthAddress> {
    return await this.txSender.getSenderAddress();
  }

  public async isItMyTurnToSubmit(): Promise<boolean> {
    const submitter = await this.txSender.getProposerAtNextEthBlock();
    const sender = await this.txSender.getSenderAddress();
    return submitter.isZero() || submitter.equals(sender);
  }

  /**
   * Publishes L2 block on L1.
   * @param block - L2 block to publish.
   * @returns True once the tx has been confirmed and is successful, false on revert or interrupt, blocks otherwise.
   */
  public async processL2Block(block: L2Block, attestations?: Attestation[]): Promise<boolean> {
    const ctx = { blockNumber: block.number, blockHash: block.hash().toString() };
    // TODO(#4148) Remove this block number check, it's here because we don't currently have proper genesis state on the contract
    const lastArchive = block.header.lastArchive.root.toBuffer();
    if (block.number != 1 && !(await this.checkLastArchiveHash(lastArchive))) {
      this.log.info(`Detected different last archive prior to publishing a block, aborting publish...`, ctx);
      return false;
    }
    const encodedBody = block.body.toBuffer();

    const processTxArgs = {
      header: block.header.toBuffer(),
      archive: block.archive.root.toBuffer(),
      body: encodedBody,
      attestations,
    };

    // Process block and publish the body if needed (if not already published)
    while (!this.interrupted) {
      let txHash;
      const timer = new Timer();

      if (await this.txSender.checkIfTxsAreAvailable(block)) {
        this.log.verbose(`Transaction effects of block ${block.number} already published.`, ctx);
        txHash = await this.sendProcessTx(processTxArgs);
      } else {
        txHash = await this.sendPublishAndProcessTx(processTxArgs);
      }

      if (!txHash) {
        this.log.info(`Failed to publish block ${block.number} to L1`, ctx);
        break;
      }

      const receipt = await this.getTransactionReceipt(txHash);
      if (!receipt) {
        this.log.info(`Failed to get receipt for tx ${txHash}`, ctx);
        break;
      }

      // Tx was mined successfully
      if (receipt.status) {
        const tx = await this.txSender.getTransactionStats(txHash);
        const stats: L1PublishBlockStats = {
          ...pick(receipt, 'gasPrice', 'gasUsed', 'transactionHash'),
          ...pick(tx!, 'calldataGas', 'calldataSize'),
          ...block.getStats(),
          eventName: 'rollup-published-to-l1',
        };
        this.log.info(`Published L2 block to L1 rollup contract`, { ...stats, ...ctx });
        this.metrics.recordProcessBlockTx(timer.ms(), stats);
        return true;
      }

      this.metrics.recordFailedTx('process');

      // Check if someone else incremented the block number
      if (!(await this.checkLastArchiveHash(lastArchive))) {
        this.log.warn('Publish failed. Detected different last archive hash.', ctx);
        break;
      }

      this.log.error(`Rollup.process tx status failed: ${receipt.transactionHash}`, ctx);
      await this.sleepOrInterrupted();
    }

    this.log.verbose('L2 block data syncing interrupted while processing blocks.', ctx);
    return false;
  }

  public async submitProof(
    header: Header,
    archiveRoot: Fr,
    proverId: Fr,
    aggregationObject: Fr[],
    proof: Proof,
  ): Promise<boolean> {
    const ctx = { blockNumber: header.globalVariables.blockNumber };

    const txArgs: L1SubmitProofArgs = {
      header: header.toBuffer(),
      archive: archiveRoot.toBuffer(),
      proverId: proverId.toBuffer(),
      aggregationObject: serializeToBuffer(aggregationObject),
      proof: proof.withoutPublicInputs(),
    };

    // Process block
    while (!this.interrupted) {
      const timer = new Timer();
      const txHash = await this.sendSubmitProofTx(txArgs);
      if (!txHash) {
        break;
      }

      const receipt = await this.getTransactionReceipt(txHash);
      if (!receipt) {
        break;
      }

      // Tx was mined successfully
      if (receipt.status) {
        const tx = await this.txSender.getTransactionStats(txHash);
        const stats: L1PublishProofStats = {
          ...pick(receipt, 'gasPrice', 'gasUsed', 'transactionHash'),
          ...pick(tx!, 'calldataGas', 'calldataSize'),
          eventName: 'proof-published-to-l1',
        };
        this.log.info(`Published L2 block to L1 rollup contract`, { ...stats, ...ctx });
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

  /**
   * Verifies that the given value of last archive in a block header equals current archive of the rollup contract
   * @param lastArchive - The last archive of the block we wish to publish.
   * @returns Boolean indicating if the hashes are equal.
   */
  private async checkLastArchiveHash(lastArchive: Buffer): Promise<boolean> {
    const fromChain = await this.txSender.getCurrentArchive();
    const areSame = lastArchive.equals(fromChain);
    if (!areSame) {
      this.log.debug(`Contract archive: ${fromChain.toString('hex')}`);
      this.log.debug(`New block last archive: ${lastArchive.toString('hex')}`);
    }
    return areSame;
  }

  private async sendSubmitProofTx(submitProofArgs: L1SubmitProofArgs): Promise<string | undefined> {
    try {
      const size = Object.values(submitProofArgs).reduce((acc, arg) => acc + arg.length, 0);
      this.log.info(`SubmitProof size=${size} bytes`);
      return await this.txSender.sendSubmitProofTx(submitProofArgs);
    } catch (err) {
      this.log.error(`Rollup submit proof failed`, err);
      return undefined;
    }
  }

  private async sendPublishTx(encodedBody: Buffer): Promise<string | undefined> {
    while (!this.interrupted) {
      try {
        this.log.info(`TxEffects size=${encodedBody.length} bytes`);
        return await this.txSender.sendPublishTx(encodedBody);
      } catch (err) {
        this.log.error(`TxEffects publish failed`, err);
        return undefined;
      }
    }
  }

  private async sendProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined> {
    while (!this.interrupted) {
      try {
        return await this.txSender.sendProcessTx(encodedData);
      } catch (err) {
        this.log.error(`Rollup publish failed`, err);
        return undefined;
      }
    }
  }

  private async sendPublishAndProcessTx(encodedData: L1ProcessArgs): Promise<string | undefined> {
    while (!this.interrupted) {
      try {
        return await this.txSender.sendPublishAndProcessTx(encodedData);
      } catch (err) {
        this.log.error(`Rollup publish failed`, err);
        return undefined;
      }
    }
  }

  private async getTransactionReceipt(txHash: string): Promise<MinimalTransactionReceipt | undefined> {
    while (!this.interrupted) {
      try {
        return await this.txSender.getTransactionReceipt(txHash);
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
