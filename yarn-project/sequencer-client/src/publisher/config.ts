import { type L1ContractAddresses, NULL_KEY } from '@aztec/ethereum';

import { type Hex } from 'viem';

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
  l1ChainId: number;

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
  l1PublishRetryIntervalMS: number;
}

export function getTxSenderConfigFromEnv(scope: 'PROVER' | 'SEQ'): Omit<TxSenderConfig, 'l1Contracts'> {
  const { ETHEREUM_HOST, L1_CHAIN_ID } = process.env;

  const PUBLISHER_PRIVATE_KEY = process.env[`${scope}_PUBLISHER_PRIVATE_KEY`];
  const REQUIRED_CONFIRMATIONS = process.env[`${scope}_REQUIRED_CONFIRMATIONS`];

  const publisherPrivateKey: Hex = PUBLISHER_PRIVATE_KEY ? `0x${PUBLISHER_PRIVATE_KEY.replace('0x', '')}` : NULL_KEY;

  return {
    rpcUrl: ETHEREUM_HOST ? ETHEREUM_HOST : '',
    requiredConfirmations: REQUIRED_CONFIRMATIONS ? +REQUIRED_CONFIRMATIONS : 1,
    publisherPrivateKey,
    l1ChainId: L1_CHAIN_ID ? +L1_CHAIN_ID : 31337,
  };
}
