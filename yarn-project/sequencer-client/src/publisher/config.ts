import { type L1ContractAddresses } from '@aztec/ethereum';

/**
 * The configuration of the rollup transaction publisher.
 */
export interface TxSenderConfig {
  /**
   * The private key to be used by the publisher.
   */
  publisherPrivateKey: `0x${string}`;

  /**
   * The RPC Url of the ethereum host.
   */
  rpcUrl: string;

  /**
   * The chain ID of the ethereum host.
   */
  l1ChainId?: number;

  /**
   * The number of confirmations required.
   */
  requiredConfirmations: number;

  /**
   * The deployed l1 contract addresses
   */
  l1Contracts: L1ContractAddresses;
}

/**
 * Configuration of the L1Publisher.
 */
export interface PublisherConfig {
  /**
   * The interval to wait between publish retries.
   */
  l1BlockPublishRetryIntervalMS: number;
}
