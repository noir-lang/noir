import { EthAddress } from '@aztec/foundation/eth-address';
import { L1Addresses } from '@aztec/types';

/**
 * There are 2 polling intervals used in this configuration. The first is the archiver polling interval, archiverPollingIntervalMS.
 * This is the interval between successive calls to eth_blockNumber via viem.
 * Results of calls to eth_blockNumber are cached by viem with this cache being updated periodically at the interval specified by viemPollingIntervalMS.
 * As a result the maximum observed polling time for new blocks will be viemPollingIntervalMS + archiverPollingIntervalMS.
 */

/**
 * The archiver configuration.
 */
export interface ArchiverConfig extends L1Addresses {
  /**
   * The url of the Ethereum RPC node.
   */
  rpcUrl: string;

  /**
   * The key for the ethereum node.
   */
  apiKey?: string;

  /**
   * The polling interval in ms for retrieving new L2 blocks and encrypted logs.
   */
  archiverPollingIntervalMS?: number;

  /**
   * The polling interval viem uses in ms
   */
  viemPollingIntervalMS?: number;

  /**
   * Eth block from which we start scanning for L2Blocks.
   */
  searchStartBlock: number;
}

/**
 * Returns the archiver configuration from the environment variables.
 * Note: If an environment variable is not set, the default value is used.
 * @returns The archiver configuration.
 */
export function getConfigEnvVars(): ArchiverConfig {
  const {
    ETHEREUM_HOST,
    ARCHIVER_POLLING_INTERVAL_MS,
    ARCHIVER_VIEM_POLLING_INTERVAL_MS,
    ROLLUP_CONTRACT_ADDRESS,
    CONTRACT_DEPLOYMENT_EMITTER_ADDRESS,
    SEARCH_START_BLOCK,
    API_KEY,
    INBOX_CONTRACT_ADDRESS,
  } = process.env;
  return {
    rpcUrl: ETHEREUM_HOST || 'http://127.0.0.1:8545/',
    archiverPollingIntervalMS: ARCHIVER_POLLING_INTERVAL_MS ? +ARCHIVER_POLLING_INTERVAL_MS : 1_000,
    viemPollingIntervalMS: ARCHIVER_VIEM_POLLING_INTERVAL_MS ? +ARCHIVER_VIEM_POLLING_INTERVAL_MS : 1_000,
    rollupContract: ROLLUP_CONTRACT_ADDRESS ? EthAddress.fromString(ROLLUP_CONTRACT_ADDRESS) : EthAddress.ZERO,
    inboxContract: INBOX_CONTRACT_ADDRESS ? EthAddress.fromString(INBOX_CONTRACT_ADDRESS) : EthAddress.ZERO,
    contractDeploymentEmitterContract: CONTRACT_DEPLOYMENT_EMITTER_ADDRESS
      ? EthAddress.fromString(CONTRACT_DEPLOYMENT_EMITTER_ADDRESS)
      : EthAddress.ZERO,
    searchStartBlock: SEARCH_START_BLOCK ? +SEARCH_START_BLOCK : 0,
    apiKey: API_KEY,
  };
}
