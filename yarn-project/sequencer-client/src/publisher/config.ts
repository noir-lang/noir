import { type L1ReaderConfig, NULL_KEY } from '@aztec/ethereum';
import { type ConfigMappingsType, getConfigFromMappings } from '@aztec/foundation/config';

/**
 * The configuration of the rollup transaction publisher.
 */
export type TxSenderConfig = L1ReaderConfig & {
  /**
   * The private key to be used by the publisher.
   */
  publisherPrivateKey: `0x${string}`;

  /**
   * The number of confirmations required.
   */
  requiredConfirmations: number;
};

/**
 * Configuration of the L1Publisher.
 */
export interface PublisherConfig {
  /**
   * The interval to wait between publish retries.
   */
  l1PublishRetryIntervalMS: number;
}

export const getTxSenderConfigMappings: (
  scope: 'PROVER' | 'SEQ',
) => ConfigMappingsType<Omit<TxSenderConfig, 'l1Contracts'>> = (scope: 'PROVER' | 'SEQ') => ({
  l1RpcUrl: {
    env: 'ETHEREUM_HOST',
    description: 'The RPC Url of the ethereum host.',
  },
  l1ChainId: {
    env: 'L1_CHAIN_ID',
    parseEnv: (val: string) => +val,
    defaultValue: 31337,
    description: 'The chain ID of the ethereum host.',
  },
  publisherPrivateKey: {
    env: `${scope}_PUBLISHER_PRIVATE_KEY`,
    description: 'The private key to be used by the publisher.',
    parseEnv: (val: string) => (val ? `0x${val.replace('0x', '')}` : NULL_KEY),
    defaultValue: NULL_KEY,
  },
  requiredConfirmations: {
    env: `${scope}_REQUIRED_CONFIRMATIONS`,
    parseEnv: (val: string) => +val,
    defaultValue: 1,
    description: 'The number of confirmations required.',
  },
});

export function getTxSenderConfigFromEnv(scope: 'PROVER' | 'SEQ'): Omit<TxSenderConfig, 'l1Contracts'> {
  return getConfigFromMappings(getTxSenderConfigMappings(scope));
}

export const getPublisherConfigMappings: (scope: 'PROVER' | 'SEQ') => ConfigMappingsType<PublisherConfig> = scope => ({
  l1PublishRetryIntervalMS: {
    env: `${scope}_PUBLISH_RETRY_INTERVAL_MS`,
    parseEnv: (val: string) => +val,
    defaultValue: 1000,
    description: 'The interval to wait between publish retries.',
  },
});

export function getPublisherConfigFromEnv(scope: 'PROVER' | 'SEQ'): PublisherConfig {
  return getConfigFromMappings(getPublisherConfigMappings(scope));
}
