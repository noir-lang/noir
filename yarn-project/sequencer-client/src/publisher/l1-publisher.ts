import { type L2Block, type Signature } from '@aztec/circuit-types';
import { type L1PublishBlockStats, type L1PublishProofStats } from '@aztec/circuit-types/stats';
import { ETHEREUM_SLOT_DURATION, EthAddress, type Header, type Proof } from '@aztec/circuits.js';
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

/**
 * @notice An attestation for the sequencing model.
 * @todo    This is not where it belongs. But I think we should do a bigger rewrite of some of
 *          this spaghetti.
 */
export type Attestation = { isEmpty: boolean; v: number; r: `0x${string}`; s: `0x${string}` };

/** Arguments to the process method of the rollup contract */
export type L1ProcessArgs = {
  /** The L2 block header. */
  header: Buffer;
  /** A root of the archive tree after the L2 block is applied. */
  archive: Buffer;
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

  constructor(config: TxSenderConfig & PublisherConfig, client: TelemetryClient) {
    this.sleepTimeMs = config?.l1PublishRetryIntervalMS ?? 60_000;
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
  }

  public getSenderAddress(): Promise<EthAddress> {
    return Promise.resolve(EthAddress.fromString(this.account.address));
  }

  // Computes who will be the L2 proposer at the next Ethereum block
  // Using next Ethereum block so we do NOT need to wait for it being mined before seeing the effect
  // @note Assumes that all ethereum slots have blocks
  async getProposerAtNextEthBlock(): Promise<EthAddress> {
    try {
      const ts = BigInt((await this.publicClient.getBlock()).timestamp + BigInt(ETHEREUM_SLOT_DURATION));
      const submitter = await this.rollupContract.read.getProposerAt([ts]);
      return EthAddress.fromString(submitter);
    } catch (err) {
      this.log.warn(`Failed to get submitter: ${err}`);
      return EthAddress.ZERO;
    }
  }

  public async isItMyTurnToSubmit(): Promise<boolean> {
    const submitter = await this.getProposerAtNextEthBlock();
    const sender = await this.getSenderAddress();
    return submitter.isZero() || submitter.equals(sender);
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

      if (await this.checkIfTxsAreAvailable(block)) {
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
        const tx = await this.getTransactionStats(txHash);
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

  async getCurrentArchive(): Promise<Buffer> {
    const archive = await this.rollupContract.read.archive();
    return Buffer.from(archive.replace('0x', ''), 'hex');
  }

  /**
   * Verifies that the given value of last archive in a block header equals current archive of the rollup contract
   * @param lastArchive - The last archive of the block we wish to publish.
   * @returns Boolean indicating if the hashes are equal.
   */
  private async checkLastArchiveHash(lastArchive: Buffer): Promise<boolean> {
    const fromChain = await this.getCurrentArchive();
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

      const { header, archive, proverId, aggregationObject, proof } = submitProofArgs;
      const args = [
        `0x${header.toString('hex')}`,
        `0x${archive.toString('hex')}`,
        `0x${proverId.toString('hex')}`,
        `0x${aggregationObject.toString('hex')}`,
        `0x${proof.toString('hex')}`,
      ] as const;

      return await this.rollupContract.write.submitProof(args, {
        account: this.account,
      });
    } catch (err) {
      this.log.error(`Rollup submit proof failed`, err);
      return undefined;
    }
  }

  private async sendPublishTx(encodedBody: Buffer): Promise<string | undefined> {
    while (!this.interrupted) {
      try {
        this.log.info(`TxEffects size=${encodedBody.length} bytes`);
        const args = [`0x${encodedBody.toString('hex')}`] as const;

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
    while (!this.interrupted) {
      try {
        if (encodedData.attestations) {
          const attestations = encodedData.attestations.map(attest => attest.toViemSignature());
          const args = [
            `0x${encodedData.header.toString('hex')}`,
            `0x${encodedData.archive.toString('hex')}`,
            attestations,
          ] as const;

          return await this.rollupContract.write.process(args, {
            account: this.account,
          });
        } else {
          const args = [`0x${encodedData.header.toString('hex')}`, `0x${encodedData.archive.toString('hex')}`] as const;

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
    while (!this.interrupted) {
      try {
        // @note  This is quite a sin, but I'm committing war crimes in this code already.
        if (encodedData.attestations) {
          const attestations = encodedData.attestations.map(attest => attest.toViemSignature());
          const args = [
            `0x${encodedData.header.toString('hex')}`,
            `0x${encodedData.archive.toString('hex')}`,
            attestations,
            `0x${encodedData.body.toString('hex')}`,
          ] as const;

          return await this.rollupContract.write.publishAndProcess(args, {
            account: this.account,
          });
        } else {
          const args = [
            `0x${encodedData.header.toString('hex')}`,
            `0x${encodedData.archive.toString('hex')}`,
            `0x${encodedData.body.toString('hex')}`,
          ] as const;

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
