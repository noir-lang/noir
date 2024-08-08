import { type L1ContractAddresses, type L1ReaderConfig, l1ReaderConfigMappings } from '@aztec/ethereum';
import { type ConfigMappingsType, getConfigFromMappings, numberConfigHelper } from '@aztec/foundation/config';

/**
 * There are 2 polling intervals used in this configuration. The first is the archiver polling interval, archiverPollingIntervalMS.
 * This is the interval between successive calls to eth_blockNumber via viem.
 * Results of calls to eth_blockNumber are cached by viem with this cache being updated periodically at the interval specified by viemPollingIntervalMS.
 * As a result the maximum observed polling time for new blocks will be viemPollingIntervalMS + archiverPollingIntervalMS.
 */

/**
 * The archiver configuration.
 */
export type ArchiverConfig = {
  /**
   * URL for an archiver service. If set, will return an archiver client as opposed to starting a new one.
   */
  archiverUrl?: string;

  /**
   * The polling interval in ms for retrieving new L2 blocks and encrypted logs.
   */
  archiverPollingIntervalMS?: number;

  /**
   * The polling interval viem uses in ms
   */
  viemPollingIntervalMS?: number;

  /**
   * The deployed L1 contract addresses
   */
  l1Contracts: L1ContractAddresses;

  /**
   * Optional dir to store data. If omitted will store in memory.
   */
  dataDirectory: string | undefined;

  /** The max number of logs that can be obtained in 1 "getUnencryptedLogs" call. */
  maxLogs?: number;
} & L1ReaderConfig;

export const archiverConfigMappings: ConfigMappingsType<ArchiverConfig> = {
  archiverUrl: {
    env: 'ARCHIVER_URL',
    description:
      'URL for an archiver service. If set, will return an archiver client as opposed to starting a new one.',
  },
  archiverPollingIntervalMS: {
    env: 'ARCHIVER_POLLING_INTERVAL_MS',
    description: 'The polling interval in ms for retrieving new L2 blocks and encrypted logs.',
    ...numberConfigHelper(1000),
  },
  viemPollingIntervalMS: {
    env: 'ARCHIVER_VIEM_POLLING_INTERVAL_MS',
    description: 'The polling interval viem uses in ms',
    ...numberConfigHelper(1000),
  },
  dataDirectory: {
    env: 'DATA_DIRECTORY',
    description: 'Optional dir to store data. If omitted will store in memory.',
  },
  maxLogs: {
    env: 'ARCHIVER_MAX_LOGS',
    description: 'The max number of logs that can be obtained in 1 "getUnencryptedLogs" call.',
    ...numberConfigHelper(1_000),
  },
  ...l1ReaderConfigMappings,
};

/**
 * Returns the archiver configuration from the environment variables.
 * Note: If an environment variable is not set, the default value is used.
 * @returns The archiver configuration.
 */
export function getArchiverConfigFromEnv(): ArchiverConfig {
  return getConfigFromMappings<ArchiverConfig>(archiverConfigMappings);
}
