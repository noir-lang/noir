import { type L2Block } from '@aztec/circuit-types';
import { ETHEREUM_SLOT_DURATION, EthAddress } from '@aztec/circuits.js';
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
  parseSignature,
} from 'viem';
import { type PrivateKeyAccount, privateKeyToAccount } from 'viem/accounts';
import * as chains from 'viem/chains';

import { type TxSenderConfig } from './config.js';
import {
  type Attestation,
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

  async attest(archive: `0x{string}`): Promise<Attestation> {
    // @note  Something seems slightly off in viem, think it should be Hex instead of Hash
    //        but as they both are just `0x${string}` it should be fine anyways.
    const signature = await this.account.signMessage({ message: { raw: archive } });
    const { r, s, v } = parseSignature(signature as `0x${string}`);

    return {
      isEmpty: false,
      v: v ? Number(v) : 0,
      r: r,
      s: s,
    };
  }

  getSenderAddress(): Promise<EthAddress> {
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
    if (encodedData.attestations) {
      const args = [
        `0x${encodedData.header.toString('hex')}`,
        `0x${encodedData.archive.toString('hex')}`,
        encodedData.attestations,
      ] as const;

      const gas = await this.rollupContract.estimateGas.process(args, {
        account: this.account,
      });
      return await this.rollupContract.write.process(args, {
        gas,
        account: this.account,
      });
    } else {
      const args = [`0x${encodedData.header.toString('hex')}`, `0x${encodedData.archive.toString('hex')}`] as const;

      const gas = await this.rollupContract.estimateGas.process(args, {
        account: this.account,
      });
      return await this.rollupContract.write.process(args, {
        gas,
        account: this.account,
      });
    }
  }

  /**
   * @notice  Publishes the body AND process the block in one transaction
   * @param encodedData - Serialized data for processing the new L2 block.
   * @returns The hash of the transaction
   */
  async sendPublishAndProcessTx(encodedData: ProcessTxArgs): Promise<string | undefined> {
    // @note  This is quite a sin, but I'm committing war crimes in this code already.
    if (encodedData.attestations) {
      const args = [
        `0x${encodedData.header.toString('hex')}`,
        `0x${encodedData.archive.toString('hex')}`,
        encodedData.attestations,
        `0x${encodedData.body.toString('hex')}`,
      ] as const;

      const gas = await this.rollupContract.estimateGas.publishAndProcess(args, {
        account: this.account,
      });
      return await this.rollupContract.write.publishAndProcess(args, {
        gas,
        account: this.account,
      });
    } else {
      const args = [
        `0x${encodedData.header.toString('hex')}`,
        `0x${encodedData.archive.toString('hex')}`,
        `0x${encodedData.body.toString('hex')}`,
      ] as const;

      const gas = await this.rollupContract.estimateGas.publishAndProcess(args, {
        account: this.account,
      });
      return await this.rollupContract.write.publishAndProcess(args, {
        gas,
        account: this.account,
      });
    }
  }

  /**
   * Sends a tx to the L1 rollup contract with a proof. Returns once the tx has been mined.
   * @param encodedData - Serialized data for the proof.
   * @returns The hash of the mined tx.
   */
  async sendSubmitProofTx(submitProofArgs: L1SubmitProofArgs): Promise<string | undefined> {
    const { header, archive, proverId, aggregationObject, proof } = submitProofArgs;
    const args = [
      `0x${header.toString('hex')}`,
      `0x${archive.toString('hex')}`,
      `0x${proverId.toString('hex')}`,
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
