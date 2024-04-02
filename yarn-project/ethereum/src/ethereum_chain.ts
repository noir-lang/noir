import { type Chain } from 'viem';

/**
 * Interface containing the connection and chain properties to interact with a blockchain.
 */
export interface EthereumChain {
  /**
   * An instance of the viem chain data.
   */
  chainInfo: Chain;

  /**
   * The actual url to be used.
   */
  rpcUrl: string;
}
