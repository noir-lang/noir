import { L1Addresses } from '@aztec/types';

/**
 * The configuration of the rollup transaction publisher.
 */
export interface TxSenderConfig extends L1Addresses {
  /**
   * The private key to be used by the publisher.
   */
  publisherPrivateKey: `0x${string}`;

  /**
   * The RPC Url of the ethereum host.
   */
  rpcUrl: string;

  /**
   * The API key of the ethereum host.
   */
  apiKey?: string;

  /**
   * The number of confirmations required.
   */
  requiredConfirmations: number;
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
