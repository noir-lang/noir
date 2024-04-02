import { AztecAddress, Fr } from '@aztec/circuits.js';
import { type L1ContractAddresses, NULL_KEY } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';

import { type Hex } from 'viem';

import { type GlobalReaderConfig } from './global_variable_builder/index.js';
import { type PublisherConfig, type TxSenderConfig } from './publisher/config.js';
import { type SequencerConfig } from './sequencer/config.js';

/** Chain configuration. */
type ChainConfig = {
  /** The chain id of the ethereum host. */
  chainId: number;
  /** The version of the rollup. */
  version: number;
};

/**
 * Configuration settings for the SequencerClient.
 */
export type SequencerClientConfig = PublisherConfig &
  TxSenderConfig &
  SequencerConfig &
  GlobalReaderConfig &
  ChainConfig;

/**
 * Creates an instance of SequencerClientConfig out of environment variables using sensible defaults for integration testing if not set.
 */
export function getConfigEnvVars(): SequencerClientConfig {
  const {
    SEQ_PUBLISHER_PRIVATE_KEY,
    ETHEREUM_HOST,
    CHAIN_ID,
    VERSION,
    API_KEY,
    SEQ_REQUIRED_CONFIRMATIONS,
    SEQ_PUBLISH_RETRY_INTERVAL_MS,
    SEQ_TX_POLLING_INTERVAL_MS,
    SEQ_MAX_TX_PER_BLOCK,
    SEQ_MIN_TX_PER_BLOCK,
    SEQ_FPC_CLASSES,
    SEQ_FPC_INSTANCES,
    AVAILABILITY_ORACLE_CONTRACT_ADDRESS,
    ROLLUP_CONTRACT_ADDRESS,
    REGISTRY_CONTRACT_ADDRESS,
    INBOX_CONTRACT_ADDRESS,
    OUTBOX_CONTRACT_ADDRESS,
    GAS_TOKEN_CONTRACT_ADDRESS,
    GAS_PORTAL_CONTRACT_ADDRESS,
    COINBASE,
    FEE_RECIPIENT,
    ACVM_WORKING_DIRECTORY,
    ACVM_BINARY_PATH,
  } = process.env;

  const publisherPrivateKey: Hex = SEQ_PUBLISHER_PRIVATE_KEY
    ? `0x${SEQ_PUBLISHER_PRIVATE_KEY.replace('0x', '')}`
    : NULL_KEY;
  // Populate the relevant addresses for use by the sequencer
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
    rpcUrl: ETHEREUM_HOST ? ETHEREUM_HOST : '',
    chainId: CHAIN_ID ? +CHAIN_ID : 31337, // 31337 is the default chain id for anvil
    version: VERSION ? +VERSION : 1, // 1 is our default version
    apiKey: API_KEY,
    requiredConfirmations: SEQ_REQUIRED_CONFIRMATIONS ? +SEQ_REQUIRED_CONFIRMATIONS : 1,
    l1BlockPublishRetryIntervalMS: SEQ_PUBLISH_RETRY_INTERVAL_MS ? +SEQ_PUBLISH_RETRY_INTERVAL_MS : 1_000,
    transactionPollingIntervalMS: SEQ_TX_POLLING_INTERVAL_MS ? +SEQ_TX_POLLING_INTERVAL_MS : 1_000,
    l1Contracts: addresses,
    publisherPrivateKey,
    maxTxsPerBlock: SEQ_MAX_TX_PER_BLOCK ? +SEQ_MAX_TX_PER_BLOCK : 32,
    minTxsPerBlock: SEQ_MIN_TX_PER_BLOCK ? +SEQ_MIN_TX_PER_BLOCK : 1,
    // TODO: undefined should not be allowed for the following 2 values in PROD
    coinbase: COINBASE ? EthAddress.fromString(COINBASE) : undefined,
    feeRecipient: FEE_RECIPIENT ? AztecAddress.fromString(FEE_RECIPIENT) : undefined,
    acvmWorkingDirectory: ACVM_WORKING_DIRECTORY ? ACVM_WORKING_DIRECTORY : undefined,
    acvmBinaryPath: ACVM_BINARY_PATH ? ACVM_BINARY_PATH : undefined,
    allowedFeePaymentContractClasses: SEQ_FPC_CLASSES ? SEQ_FPC_CLASSES.split(',').map(Fr.fromString) : [],
    allowedFeePaymentContractInstances: SEQ_FPC_INSTANCES
      ? SEQ_FPC_INSTANCES.split(',').map(AztecAddress.fromString)
      : [],
  };
}
