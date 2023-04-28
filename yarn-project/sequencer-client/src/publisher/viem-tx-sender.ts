import { TxSenderConfig } from './config.js';
import { L1ProcessArgs as ProcessTxArgs, L1PublisherTxSender } from './l1-publisher.js';
import { ContractPublicData, UnverifiedData } from '@aztec/types';
import { createDebugLogger } from '@aztec/foundation';
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
import { RollupAbi, UnverifiedDataEmitterAbi } from '@aztec/l1-contracts/viem';
import { PrivateKeyAccount, privateKeyToAccount } from 'viem/accounts';
import * as chains from 'viem/chains';

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
      chainId,
      publisherPrivateKey,
      rollupContract: rollupContractAddress,
      unverifiedDataEmitterContract: unverifiedDataEmitterContractAddress,
    } = config;

    this.account = privateKeyToAccount(`0x${publisherPrivateKey.toString('hex')}`);
    const chain = this.getChain(chainId);
    const walletClient = createWalletClient({
      account: this.account,
      chain,
      transport: http(rpcUrl),
    });

    this.publicClient = createPublicClient({
      chain: chain,
      transport: http(rpcUrl),
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

  async getTransactionReceipt(txHash: string): Promise<{ status: boolean; transactionHash: string } | undefined> {
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

  async sendEmitUnverifiedDataTx(l2BlockNum: number, unverifiedData: UnverifiedData): Promise<string | undefined> {
    const args = [BigInt(l2BlockNum), `0x${unverifiedData.toBuffer().toString('hex')}`] as const;

    const gas = await this.unverifiedDataEmitterContract.estimateGas.emitUnverifiedData(args, {
      account: this.account,
    });
    const hash = await this.unverifiedDataEmitterContract.write.emitUnverifiedData(args, {
      account: this.account,
      gas,
    });
    return hash;
  }

  async sendEmitContractDeploymentTx(
    l2BlockNum: number,
    newContractData: ContractPublicData[],
  ): Promise<string | undefined> {
    for (const contractPublicData of newContractData) {
      const args = [
        BigInt(l2BlockNum),
        contractPublicData.contractData.contractAddress.toString() as Hex,
        contractPublicData.contractData.portalContractAddress.toString() as Hex,
        `0x${contractPublicData.bytecode.toString('hex')}`,
      ] as const;

      const gas = await this.unverifiedDataEmitterContract.estimateGas.emitContractDeployment(args, {
        account: this.account,
      });
      const hash = await this.unverifiedDataEmitterContract.write.emitContractDeployment(args, {
        gas,
        account: this.account,
      });
      return hash;
    }
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
