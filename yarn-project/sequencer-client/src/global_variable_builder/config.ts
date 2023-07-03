import { EthAddress } from '@aztec/circuits.js';

/**
 * Configuration of the L1GlobalReader.
 */
export interface GlobalReaderConfig {
  /**
   * Rollup contract address.
   */
  rollupContract: EthAddress;
  /**
   * The RPC Url of the ethereum host.
   */
  rpcUrl: string;
  /**
   * The API key of the ethereum host.
   */
  apiKey?: string;
}
