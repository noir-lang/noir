import { TxSenderConfig } from './config.js';
import { L1ProcessArgs as ProcessTxArgs, L1PublisherTxSender, MinimalTransactionReceipt } from './l1-publisher.js';
import { ContractPublicData, UnverifiedData } from '@aztec/types';

import {
  GetContractReturnType,
  Hex,
  HttpTransport,
  PublicClient,
  WalletClient,
  createPublicClient,
  createWalletClient,
  getAddress,
  getContract,
  http,
} from 'viem';
import { RollupAbi, UnverifiedDataEmitterAbi } from '@aztec/l1-artifacts';
import { PrivateKeyAccount, privateKeyToAccount } from 'viem/accounts';
import * as chains from 'viem/chains';
import { createDebugLogger } from '@aztec/foundation/log';
import { createEthereumChain } from '@aztec/ethereum';

/**
 * Pushes transactions to the L1 rollup contract using viem.
 */
export class ViemTxSender implements L1PublisherTxSender {
  private rollupContract: GetContractReturnType<
    typeof RollupAbi,
    PublicClient<HttpTransport, chains.Chain>,
    WalletClient<HttpTransport, chains.Chain, PrivateKeyAccount>
  >;
  private unverifiedDataEmitterContract: GetContractReturnType<
    typeof UnverifiedDataEmitterAbi,
    PublicClient<HttpTransport, chains.Chain>,
    WalletClient<HttpTransport, chains.Chain, PrivateKeyAccount>
  >;

  private log = createDebugLogger('aztec:sequencer:viem-tx-sender');
  private publicClient: PublicClient<HttpTransport, chains.Chain>;
  private account: PrivateKeyAccount;

  constructor(config: TxSenderConfig) {
    const {
      rpcUrl,
      apiKey,
      publisherPrivateKey,
      rollupContract: rollupContractAddress,
      unverifiedDataEmitterContract: unverifiedDataEmitterContractAddress,
    } = config;
    const chain = createEthereumChain(rpcUrl, apiKey);
    this.account = privateKeyToAccount(`0x${publisherPrivateKey.toString('hex')}`);
    const walletClient = createWalletClient({
      account: this.account,
      chain: chain.chainInfo,
      transport: http(chain.rpcUrl),
    });

    this.publicClient = createPublicClient({
      chain: chain.chainInfo,
      transport: http(chain.rpcUrl),
    });

    this.rollupContract = getContract({
      address: getAddress(rollupContractAddress.toString()),
      abi: RollupAbi,
      publicClient: this.publicClient,
      walletClient,
    });
    this.unverifiedDataEmitterContract = getContract({
      address: getAddress(unverifiedDataEmitterContractAddress.toString()),
      abi: UnverifiedDataEmitterAbi,
      publicClient: this.publicClient,
      walletClient,
    });
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

    // TODO: check for confirmations

    if (receipt) {
      return {
        status: receipt.status === 'success',
        transactionHash: txHash,
      };
    }

    this.log('Receipt not found for tx hash', txHash);
    return undefined;
  }

  /**
   * Sends a tx to the L1 rollup contract with a new L2 block. Returns once the tx has been mined.
   * @param encodedData - Serialized data for processing the new L2 block.
   * @returns The hash of the mined tx.
   */
  async sendProcessTx(encodedData: ProcessTxArgs): Promise<string | undefined> {
    const args = [`0x${encodedData.proof.toString('hex')}`, `0x${encodedData.inputs.toString('hex')}`] as const;

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
   * Sends a tx to the unverified data emitter contract with unverified data. Returns once the tx has been mined.
   * @param l2BlockNum - Number of the L2 block that owns this unverified data.
   * @param l2BlockHash - The hash of the block corresponding to this data.
   * @param unverifiedData - Data to publish.
   * @returns The hash of the mined tx.
   */
  async sendEmitUnverifiedDataTx(
    l2BlockNum: number,
    l2BlockHash: Buffer,
    unverifiedData: UnverifiedData,
  ): Promise<string | undefined> {
    const args = [
      BigInt(l2BlockNum),
      `0x${l2BlockHash.toString('hex')}`,
      `0x${unverifiedData.toBuffer().toString('hex')}`,
    ] as const;

    const gas = await this.unverifiedDataEmitterContract.estimateGas.emitUnverifiedData(args, {
      account: this.account,
    });
    const hash = await this.unverifiedDataEmitterContract.write.emitUnverifiedData(args, {
      account: this.account,
      gas,
    });
    return hash;
  }

  /**
   * Sends a tx to the unverified data emitter contract with contract deployment data such as bytecode. Returns once the tx has been mined.
   * @param l2BlockNum - Number of the L2 block that owns this unverified data.
   * @param l2BlockHash - The hash of the block corresponding to this data.
   * @param newContractData - Data to publish.
   * @returns The hash of the mined tx.
   */
  async sendEmitContractDeploymentTx(
    l2BlockNum: number,
    l2BlockHash: Buffer,
    newContractData: ContractPublicData[],
  ): Promise<(string | undefined)[]> {
    const hashes: string[] = [];
    for (const contractPublicData of newContractData) {
      const args = [
        BigInt(l2BlockNum),
        contractPublicData.contractData.contractAddress.toString() as Hex,
        contractPublicData.contractData.portalContractAddress.toString() as Hex,
        `0x${l2BlockHash.toString('hex')}`,
        `0x${contractPublicData.bytecode.toString('hex')}`,
      ] as const;

      const gas = await this.unverifiedDataEmitterContract.estimateGas.emitContractDeployment(args, {
        account: this.account,
      });
      const hash = await this.unverifiedDataEmitterContract.write.emitContractDeployment(args, {
        gas,
        account: this.account,
      });
      hashes.push(hash);
    }
    return hashes;
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
