import { createEthereumChain } from '@aztec/ethereum';
import { RollupAbi } from '@aztec/l1-artifacts';
import {
  GetContractReturnType,
  PublicClient,
  HttpTransport,
  createPublicClient,
  http,
  getContract,
  getAddress,
} from 'viem';
import * as chains from 'viem/chains';
import { GlobalReaderConfig } from './config.js';
import { L1GlobalReader } from './global_builder.js';

/**
 * Reads values from L1 state using viem.
 */
export class ViemReader implements L1GlobalReader {
  private rollupContract: GetContractReturnType<typeof RollupAbi, PublicClient<HttpTransport, chains.Chain>>;
  private publicClient: PublicClient<HttpTransport, chains.Chain>;

  constructor(config: GlobalReaderConfig) {
    const { rpcUrl, apiKey, rollupContract: rollupContractAddress } = config;

    const chain = createEthereumChain(rpcUrl, apiKey);

    this.publicClient = createPublicClient({
      chain: chain.chainInfo,
      transport: http(chain.rpcUrl),
    });

    this.rollupContract = getContract({
      address: getAddress(rollupContractAddress.toString()),
      abi: RollupAbi,
      publicClient: this.publicClient,
    });
  }

  /**
   * Fetches the last timestamp that a block was processed by the contract.
   * @returns The last timestamp that a block was processed by the contract.
   */
  public async getLastTimestamp(): Promise<bigint> {
    return BigInt(await this.rollupContract.read.lastBlockTs());
  }

  /**
   * Fetches the version of the rollup contract.
   * @returns The version of the rollup contract.
   */
  public async getVersion(): Promise<bigint> {
    return BigInt(await this.rollupContract.read.VERSION());
  }

  /**
   * Gets the chain id.
   * @returns The chain id.
   */
  public async getChainId(): Promise<bigint> {
    return await Promise.resolve(BigInt(this.publicClient.chain.id));
  }
}
