import { type L2Block } from '@aztec/circuit-types';
import { EthAddress } from '@aztec/circuits.js';
import { createEthereumChain } from '@aztec/ethereum';
import { createDebugLogger } from '@aztec/foundation/log';
import { AvailabilityOracleAbi, RollupAbi } from '@aztec/l1-artifacts';

import {
  type GetContractReturnType,
  type Hex,
  type HttpTransport,
  type PublicClient,
  type WalletClient,
  createPublicClient,
  createWalletClient,
  getAddress,
  getContract,
  hexToBytes,
  http,
} from 'viem';
import { type PrivateKeyAccount, privateKeyToAccount } from 'viem/accounts';
import * as chains from 'viem/chains';

import { type TxSenderConfig } from './config.js';
import {
  type L1PublisherTxSender,
  type L1SubmitProofArgs,
  type MinimalTransactionReceipt,
  type L1ProcessArgs as ProcessTxArgs,
  type TransactionStats,
} from './l1-publisher.js';

/**
 * Pushes transactions to the L1 rollup contract using viem.
 */
export class ViemTxSender implements L1PublisherTxSender {
  private availabilityOracleContract: GetContractReturnType<
    typeof AvailabilityOracleAbi,
    WalletClient<HttpTransport, chains.Chain, PrivateKeyAccount>
  >;
  private rollupContract: GetContractReturnType<
    typeof RollupAbi,
    WalletClient<HttpTransport, chains.Chain, PrivateKeyAccount>
  >;

  private log = createDebugLogger('aztec:sequencer:viem-tx-sender');
  private publicClient: PublicClient<HttpTransport, chains.Chain>;
  private account: PrivateKeyAccount;

  constructor(config: TxSenderConfig) {
    const { rpcUrl, l1ChainId: chainId, publisherPrivateKey, l1Contracts } = config;
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

  getSenderAddress(): Promise<EthAddress> {
    return Promise.resolve(EthAddress.fromString(this.account.address));
  }

  async getSubmitterAddressForBlock(blockNumber: number): Promise<EthAddress> {
    try {
      const submitter = await this.rollupContract.read.whoseTurnIsIt([BigInt(blockNumber)]);
      return EthAddress.fromString(submitter);
    } catch (err) {
      this.log.warn(`Failed to get submitter for block ${blockNumber}: ${err}`);
      return EthAddress.ZERO;
    }
  }

  async getCurrentArchive(): Promise<Buffer> {
    const archive = await this.rollupContract.read.archive();
    return Buffer.from(archive.replace('0x', ''), 'hex');
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
   * Returns a tx receipt if the tx has been mined.
   * @param txHash - Hash of the tx to look for.
   * @returns Undefined if the tx hasn't been mined yet, the receipt otherwise.
   */
  async getTransactionReceipt(txHash: string): Promise<MinimalTransactionReceipt | undefined> {
    const receipt = await this.publicClient.getTransactionReceipt({
      hash: txHash as Hex,
    });

    if (receipt) {
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
  }

  /**
   * Publishes tx effects to Availability Oracle.
   * @param encodedBody - Encoded block body.
   * @returns The hash of the mined tx.
   */
  async sendPublishTx(encodedBody: Buffer): Promise<string | undefined> {
    const args = [`0x${encodedBody.toString('hex')}`] as const;

    const gas = await this.availabilityOracleContract.estimateGas.publish(args, {
      account: this.account,
    });
    const hash = await this.availabilityOracleContract.write.publish(args, {
      gas,
      account: this.account,
    });
    return hash;
  }

  /**
   * Sends a tx to the L1 rollup contract with a new L2 block. Returns once the tx has been mined.
   * @param encodedData - Serialized data for processing the new L2 block.
   * @returns The hash of the mined tx.
   */
  async sendProcessTx(encodedData: ProcessTxArgs): Promise<string | undefined> {
    const args = [`0x${encodedData.header.toString('hex')}`, `0x${encodedData.archive.toString('hex')}`] as const;

    const gas = await this.rollupContract.estimateGas.process(args, {
      account: this.account,
    });
    const hash = await this.rollupContract.write.process(args, {
      gas,
      account: this.account,
    });
    return hash;
  }

  /**
   * Sends a tx to the L1 rollup contract with a proof. Returns once the tx has been mined.
   * @param encodedData - Serialized data for the proof.
   * @returns The hash of the mined tx.
   */
  async sendSubmitProofTx(submitProofArgs: L1SubmitProofArgs): Promise<string | undefined> {
    const { header, archive, aggregationObject, proof } = submitProofArgs;
    const args = [
      `0x${header.toString('hex')}`,
      `0x${archive.toString('hex')}`,
      `0x${aggregationObject.toString('hex')}`,
      `0x${proof.toString('hex')}`,
    ] as const;

    const gas = await this.rollupContract.estimateGas.submitProof(args, {
      account: this.account,
    });
    const hash = await this.rollupContract.write.submitProof(args, {
      gas,
      account: this.account,
    });

    return hash;
  }

  /**
   * Gets the chain object for the given chain id.
   * @param chainId - Chain id of the target EVM chain.
   * @returns Viem's chain object.
   */
  private getChain(chainId: number) {
    for (const chain of Object.values(chains)) {
      if ('id' in chain && chain.id === chainId) {
        return chain;
      }
    }

    throw new Error(`Chain with id ${chainId} not found`);
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
