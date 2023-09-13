import { createEthereumChain } from '@aztec/ethereum';
import { createDebugLogger } from '@aztec/foundation/log';
import { ContractDeploymentEmitterAbi, RollupAbi } from '@aztec/l1-artifacts';
import { BLOB_SIZE_IN_BYTES, ExtendedContractData } from '@aztec/types';

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
import { PrivateKeyAccount, privateKeyToAccount } from 'viem/accounts';
import * as chains from 'viem/chains';

import { TxSenderConfig } from './config.js';
import { L1PublisherTxSender, MinimalTransactionReceipt, L1ProcessArgs as ProcessTxArgs } from './l1-publisher.js';

/**
 * Pushes transactions to the L1 rollup contract using viem.
 */
export class ViemTxSender implements L1PublisherTxSender {
  private rollupContract: GetContractReturnType<
    typeof RollupAbi,
    PublicClient<HttpTransport, chains.Chain>,
    WalletClient<HttpTransport, chains.Chain, PrivateKeyAccount>
  >;
  private contractDeploymentEmitterContract: GetContractReturnType<
    typeof ContractDeploymentEmitterAbi,
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
      contractDeploymentEmitterContract: contractDeploymentEmitterContractAddress,
    } = config;
    const chain = createEthereumChain(rpcUrl, apiKey);
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

    this.rollupContract = getContract({
      address: getAddress(rollupContractAddress.toString()),
      abi: RollupAbi,
      publicClient: this.publicClient,
      walletClient,
    });
    this.contractDeploymentEmitterContract = getContract({
      address: getAddress(contractDeploymentEmitterContractAddress.toString()),
      abi: ContractDeploymentEmitterAbi,
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
   * Sends a tx to the contract deployment emitter contract with contract deployment data such as bytecode. Returns once the tx has been mined.
   * @param l2BlockNum - Number of the L2 block that owns this encrypted logs.
   * @param l2BlockHash - The hash of the block corresponding to this data.
   * @param newExtendedContractData - Data to publish.
   * @returns The hash of the mined tx.
   */
  async sendEmitContractDeploymentTx(
    l2BlockNum: number,
    l2BlockHash: Buffer,
    newExtendedContractData: ExtendedContractData[],
  ): Promise<(string | undefined)[]> {
    const hashes: string[] = [];
    for (const extendedContractData of newExtendedContractData) {
      const args = [
        BigInt(l2BlockNum),
        extendedContractData.contractData.contractAddress.toString() as Hex,
        extendedContractData.contractData.portalContractAddress.toString() as Hex,
        `0x${l2BlockHash.toString('hex')}`,
        extendedContractData.partialAddress.toString(true),
        extendedContractData.publicKey.x.toString(true),
        extendedContractData.publicKey.y.toString(true),
        `0x${extendedContractData.bytecode.toString('hex')}`,
      ] as const;

      const codeSize = extendedContractData.bytecode.length;
      this.log(`Bytecode is ${codeSize} bytes and require ${codeSize / BLOB_SIZE_IN_BYTES} blobs`);

      const gas = await this.contractDeploymentEmitterContract.estimateGas.emitContractDeployment(args, {
        account: this.account,
      });
      const hash = await this.contractDeploymentEmitterContract.write.emitContractDeployment(args, {
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
