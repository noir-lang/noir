import { L1Addresses } from '@aztec/l1-contracts';

/**
 * The archiver configuration.
 */
export interface ArchiverConfig extends L1Addresses {
  /**
   * The url of the Ethereum RPC node.
   */
  rpcUrl: string;

  /**
   * The polling interval in ms for retrieving new L2 blocks and unverified data.
   */
  archiverPollingInterval?: number;
}
