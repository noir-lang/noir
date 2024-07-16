import { type AllowedElement } from '@aztec/circuit-types';
import { AztecAddress, Fr, FunctionSelector, getContractClassFromArtifact } from '@aztec/circuits.js';
import { type L1ContractAddresses, NULL_KEY } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { FPCContract } from '@aztec/noir-contracts.js/FPC';
import { TokenContractArtifact } from '@aztec/noir-contracts.js/Token';
import { AuthRegistryAddress } from '@aztec/protocol-contracts/auth-registry';
import { GasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { type Hex } from 'viem';

import { type GlobalReaderConfig } from './global_variable_builder/index.js';
import { type PublisherConfig, type TxSenderConfig } from './publisher/config.js';
import { type SequencerConfig } from './sequencer/config.js';

/** Chain configuration. */
type ChainConfig = {
  /** The chain id of the ethereum host. */
  l1ChainId: number;
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
    L1_CHAIN_ID,
    VERSION,
    SEQ_REQUIRED_CONFIRMATIONS,
    SEQ_PUBLISH_RETRY_INTERVAL_MS,
    SEQ_TX_POLLING_INTERVAL_MS,
    SEQ_MAX_TX_PER_BLOCK,
    SEQ_MIN_TX_PER_BLOCK,
    SEQ_ALLOWED_SETUP_FN,
    SEQ_ALLOWED_TEARDOWN_FN,
    SEQ_MAX_BLOCK_SIZE_IN_BYTES,
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
    ENFORCE_FEES = '',
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
    enforceFees: ['1', 'true'].includes(ENFORCE_FEES),
    rpcUrl: ETHEREUM_HOST ? ETHEREUM_HOST : '',
    l1ChainId: L1_CHAIN_ID ? +L1_CHAIN_ID : 31337, // 31337 is the default chain id for anvil
    version: VERSION ? +VERSION : 1, // 1 is our default version
    requiredConfirmations: SEQ_REQUIRED_CONFIRMATIONS ? +SEQ_REQUIRED_CONFIRMATIONS : 1,
    l1BlockPublishRetryIntervalMS: SEQ_PUBLISH_RETRY_INTERVAL_MS ? +SEQ_PUBLISH_RETRY_INTERVAL_MS : 1_000,
    transactionPollingIntervalMS: SEQ_TX_POLLING_INTERVAL_MS ? +SEQ_TX_POLLING_INTERVAL_MS : 1_000,
    maxBlockSizeInBytes: SEQ_MAX_BLOCK_SIZE_IN_BYTES ? +SEQ_MAX_BLOCK_SIZE_IN_BYTES : undefined,
    l1Contracts: addresses,
    publisherPrivateKey,
    maxTxsPerBlock: SEQ_MAX_TX_PER_BLOCK ? +SEQ_MAX_TX_PER_BLOCK : 32,
    minTxsPerBlock: SEQ_MIN_TX_PER_BLOCK ? +SEQ_MIN_TX_PER_BLOCK : 1,
    // TODO: undefined should not be allowed for the following 2 values in PROD
    coinbase: COINBASE ? EthAddress.fromString(COINBASE) : undefined,
    feeRecipient: FEE_RECIPIENT ? AztecAddress.fromString(FEE_RECIPIENT) : undefined,
    acvmWorkingDirectory: ACVM_WORKING_DIRECTORY ? ACVM_WORKING_DIRECTORY : undefined,
    acvmBinaryPath: ACVM_BINARY_PATH ? ACVM_BINARY_PATH : undefined,
    allowedInSetup: SEQ_ALLOWED_SETUP_FN
      ? parseSequencerAllowList(SEQ_ALLOWED_SETUP_FN)
      : getDefaultAllowedSetupFunctions(),
    allowedInTeardown: SEQ_ALLOWED_TEARDOWN_FN
      ? parseSequencerAllowList(SEQ_ALLOWED_TEARDOWN_FN)
      : getDefaultAllowedTeardownFunctions(),
  };
}

/**
 * Parses a string to a list of allowed elements.
 * Each encoded is expected to be of one of the following formats
 * `I:${address}`
 * `I:${address}:${selector}`
 * `C:${classId}`
 * `C:${classId}:${selector}`
 *
 * @param value The string to parse
 * @returns A list of allowed elements
 */
export function parseSequencerAllowList(value: string): AllowedElement[] {
  const entries: AllowedElement[] = [];

  if (!value) {
    return entries;
  }

  for (const val of value.split(',')) {
    const [typeString, identifierString, selectorString] = val.split(':');
    const selector = selectorString !== undefined ? FunctionSelector.fromString(selectorString) : undefined;

    if (typeString === 'I') {
      if (selector) {
        entries.push({
          address: AztecAddress.fromString(identifierString),
          selector,
        });
      } else {
        entries.push({
          address: AztecAddress.fromString(identifierString),
        });
      }
    } else if (typeString === 'C') {
      if (selector) {
        entries.push({
          classId: Fr.fromString(identifierString),
          selector,
        });
      } else {
        entries.push({
          classId: Fr.fromString(identifierString),
        });
      }
    }
  }

  return entries;
}

function getDefaultAllowedSetupFunctions(): AllowedElement[] {
  return [
    // needed for authwit support
    {
      address: AuthRegistryAddress,
    },
    // needed for claiming on the same tx as a spend
    {
      address: GasTokenAddress,
      selector: FunctionSelector.fromSignature('_increase_public_balance((Field),Field)'),
    },
    // needed for private transfers via FPC
    {
      classId: getContractClassFromArtifact(TokenContractArtifact).id,
      selector: FunctionSelector.fromSignature('_increase_public_balance((Field),Field)'),
    },
    {
      classId: getContractClassFromArtifact(FPCContract.artifact).id,
      selector: FunctionSelector.fromSignature('prepare_fee((Field),Field,(Field),Field)'),
    },
  ];
}

function getDefaultAllowedTeardownFunctions(): AllowedElement[] {
  return [
    {
      classId: getContractClassFromArtifact(FPCContract.artifact).id,
      selector: FunctionSelector.fromSignature('pay_refund((Field),Field,(Field))'),
    },
    {
      classId: getContractClassFromArtifact(FPCContract.artifact).id,
      selector: FunctionSelector.fromSignature('pay_refund_with_shielded_rebate(Field,(Field),Field)'),
    },
  ];
}
