import { type AllowedFunction } from '@aztec/circuit-types';
import { AztecAddress, Fr, FunctionSelector, getContractClassFromArtifact } from '@aztec/circuits.js';
import { type L1ContractAddresses, NULL_KEY } from '@aztec/ethereum';
import { EthAddress } from '@aztec/foundation/eth-address';
import { EcdsaAccountContractArtifact } from '@aztec/noir-contracts.js/EcdsaAccount';
import { FPCContract } from '@aztec/noir-contracts.js/FPC';
import { GasTokenContract } from '@aztec/noir-contracts.js/GasToken';
import { SchnorrAccountContractArtifact } from '@aztec/noir-contracts.js/SchnorrAccount';
import { SchnorrHardcodedAccountContractArtifact } from '@aztec/noir-contracts.js/SchnorrHardcodedAccount';
import { SchnorrSingleKeyAccountContractArtifact } from '@aztec/noir-contracts.js/SchnorrSingleKeyAccount';
import { TokenContractArtifact } from '@aztec/noir-contracts.js/Token';

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
    SEQ_ALLOWED_SETUP_FN,
    SEQ_ALLOWED_TEARDOWN_FN,
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
    allowedFunctionsInSetup: SEQ_ALLOWED_SETUP_FN
      ? parseSequencerAllowList(SEQ_ALLOWED_SETUP_FN)
      : getDefaultAllowedSetupFunctions(),
    allowedFunctionsInTeardown: SEQ_ALLOWED_TEARDOWN_FN
      ? parseSequencerAllowList(SEQ_ALLOWED_TEARDOWN_FN)
      : getDefaultAllowedTeardownFunctions(),
  };
}

function parseSequencerAllowList(value: string): AllowedFunction[] {
  const entries: AllowedFunction[] = [];

  if (!value) {
    return entries;
  }

  for (const val of value.split(',')) {
    const [identifierString, selectorString] = val.split(':');
    const selector = FunctionSelector.fromString(selectorString);

    if (identifierString.startsWith('0x')) {
      entries.push({
        address: AztecAddress.fromString(identifierString),
        selector,
      });
    } else {
      entries.push({
        classId: Fr.fromString(identifierString),
        selector,
      });
    }
  }

  return entries;
}

function getDefaultAllowedSetupFunctions(): AllowedFunction[] {
  return [
    {
      classId: getContractClassFromArtifact(SchnorrAccountContractArtifact).id,
      selector: FunctionSelector.fromSignature('approve_public_authwit(Field)'),
    },
    {
      classId: getContractClassFromArtifact(SchnorrHardcodedAccountContractArtifact).id,
      selector: FunctionSelector.fromSignature('approve_public_authwit(Field)'),
    },
    {
      classId: getContractClassFromArtifact(SchnorrSingleKeyAccountContractArtifact).id,
      selector: FunctionSelector.fromSignature('approve_public_authwit(Field)'),
    },
    {
      classId: getContractClassFromArtifact(EcdsaAccountContractArtifact).id,
      selector: FunctionSelector.fromSignature('approve_public_authwit(Field)'),
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

function getDefaultAllowedTeardownFunctions(): AllowedFunction[] {
  return [
    {
      classId: getContractClassFromArtifact(GasTokenContract.artifact).id,
      selector: FunctionSelector.fromSignature('pay_fee(Field)'),
    },
    {
      classId: getContractClassFromArtifact(FPCContract.artifact).id,
      selector: FunctionSelector.fromSignature('pay_fee((Field),Field,(Field))'),
    },
    {
      classId: getContractClassFromArtifact(FPCContract.artifact).id,
      selector: FunctionSelector.fromSignature('pay_fee_with_shielded_rebate(Field,(Field),Field)'),
    },
  ];
}
