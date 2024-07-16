import { type L1ContractAddresses } from '@aztec/ethereum';

/**
 * Configuration of the L1GlobalReader.
 */
export interface GlobalReaderConfig {
  /**
   * The RPC Url of the ethereum host.
   */
  rpcUrl: string;
  /**
   * The chain ID of the ethereum host.
   */
  l1ChainId: number;

  /**
   * The deployed l1 contract addresses
   */
  l1Contracts: L1ContractAddresses;
}
