import { L1ContractAddresses } from '@aztec/ethereum';

/**
 * Configuration of the L1GlobalReader.
 */
export interface GlobalReaderConfig {
  /**
   * The RPC Url of the ethereum host.
   */
  rpcUrl: string;
  /**
   * The API key of the ethereum host.
   */
  apiKey?: string;

  /**
   * The deployed l1 contract addresses
   */
  l1Contracts: L1ContractAddresses;
}
