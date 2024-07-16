import { type L1ContractAddresses } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';

/**
 * There are 2 polling intervals used in this configuration. The first is the archiver polling interval, archiverPollingIntervalMS.
 * This is the interval between successive calls to eth_blockNumber via viem.
 * Results of calls to eth_blockNumber are cached by viem with this cache being updated periodically at the interval specified by viemPollingIntervalMS.
 * As a result the maximum observed polling time for new blocks will be viemPollingIntervalMS + archiverPollingIntervalMS.
 */

/**
 * The archiver configuration.
 */
export interface ArchiverConfig {
  /**
   * The url of the Ethereum RPC node.
   */
  rpcUrl: string;

  /**
   * The key for the ethereum node.
   */
  apiKey?: string;

  /**
   * The L1 chain's ID
   */
  l1ChainId?: number;

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
  dataDirectory?: string;

  /** The max number of logs that can be obtained in 1 "getUnencryptedLogs" call. */
  maxLogs?: number;
}

/**
 * Returns the archiver configuration from the environment variables.
 * Note: If an environment variable is not set, the default value is used.
 * @returns The archiver configuration.
 */
export function getConfigEnvVars(): ArchiverConfig {
  const {
    ETHEREUM_HOST,
    L1_CHAIN_ID,
    ARCHIVER_POLLING_INTERVAL_MS,
    ARCHIVER_VIEM_POLLING_INTERVAL_MS,
    AVAILABILITY_ORACLE_CONTRACT_ADDRESS,
    ROLLUP_CONTRACT_ADDRESS,
    API_KEY,
    INBOX_CONTRACT_ADDRESS,
    OUTBOX_CONTRACT_ADDRESS,
    REGISTRY_CONTRACT_ADDRESS,
    GAS_TOKEN_CONTRACT_ADDRESS,
    GAS_PORTAL_CONTRACT_ADDRESS,
    DATA_DIRECTORY,
  } = process.env;
  // Populate the relevant addresses for use by the archiver.
  const addresses: L1ContractAddresses = {
    availabilityOracleAddress: AVAILABILITY_ORACLE_CONTRACT_ADDRESS
      ? EthAddress.fromString(AVAILABILITY_ORACLE_CONTRACT_ADDRESS)
      : EthAddress.ZERO,
    rollupAddress: ROLLUP_CONTRACT_ADDRESS ? EthAddress.fromString(ROLLUP_CONTRACT_ADDRESS) : EthAddress.ZERO,
    registryAddress: REGISTRY_CONTRACT_ADDRESS ? EthAddress.fromString(REGISTRY_CONTRACT_ADDRESS) : EthAddress.ZERO,
    inboxAddress: INBOX_CONTRACT_ADDRESS ? EthAddress.fromString(INBOX_CONTRACT_ADDRESS) : EthAddress.ZERO,
    outboxAddress: OUTBOX_CONTRACT_ADDRESS ? EthAddress.fromString(OUTBOX_CONTRACT_ADDRESS) : EthAddress.ZERO,
    gasTokenAddress: GAS_TOKEN_CONTRACT_ADDRESS ? EthAddress.fromString(GAS_TOKEN_CONTRACT_ADDRESS) : EthAddress.ZERO,
    gasPortalAddress: GAS_PORTAL_CONTRACT_ADDRESS
      ? EthAddress.fromString(GAS_PORTAL_CONTRACT_ADDRESS)
      : EthAddress.ZERO,
  };
  return {
    rpcUrl: ETHEREUM_HOST || '',
    l1ChainId: L1_CHAIN_ID ? +L1_CHAIN_ID : 31337, // 31337 is the default chain id for anvil
    archiverPollingIntervalMS: ARCHIVER_POLLING_INTERVAL_MS ? +ARCHIVER_POLLING_INTERVAL_MS : 1_000,
    viemPollingIntervalMS: ARCHIVER_VIEM_POLLING_INTERVAL_MS ? +ARCHIVER_VIEM_POLLING_INTERVAL_MS : 1_000,
    apiKey: API_KEY,
    l1Contracts: addresses,
    dataDirectory: DATA_DIRECTORY,
  };
}
