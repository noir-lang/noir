import { createEthereumChain } from '@aztec/ethereum';
import { RollupAbi } from '@aztec/l1-artifacts';

import {
  GetContractReturnType,
  HttpTransport,
  PublicClient,
  createPublicClient,
  getAddress,
  getContract,
  http,
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

  public async getLastTimestamp(): Promise<bigint> {
    return BigInt(await this.rollupContract.read.lastBlockTs());
  }

  public async getVersion(): Promise<bigint> {
    return BigInt(await this.rollupContract.read.VERSION());
  }

  public async getChainId(): Promise<bigint> {
    return await Promise.resolve(BigInt(this.publicClient.chain.id));
  }

  public async getL1CurrentTime(): Promise<bigint> {
    return await Promise.resolve((await this.publicClient.getBlock()).timestamp);
  }

  public async getLastWarpedBlockTs(): Promise<bigint> {
    return BigInt(await this.rollupContract.read.lastWarpedBlockTs());
  }
}
