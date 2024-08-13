import { type AllowedElement } from '@aztec/circuit-types';
import { AztecAddress, Fr, FunctionSelector, getContractClassFromArtifact } from '@aztec/circuits.js';
import { type L1ReaderConfig, l1ReaderConfigMappings } from '@aztec/ethereum';
import {
  type ConfigMappingsType,
  booleanConfigHelper,
  getConfigFromMappings,
  numberConfigHelper,
} from '@aztec/foundation/config';
import { EthAddress } from '@aztec/foundation/eth-address';
import { FPCContract } from '@aztec/noir-contracts.js/FPC';
import { TokenContractArtifact } from '@aztec/noir-contracts.js/Token';
import { AuthRegistryAddress } from '@aztec/protocol-contracts/auth-registry';
import { FeeJuiceAddress } from '@aztec/protocol-contracts/fee-juice';

import {
  type PublisherConfig,
  type TxSenderConfig,
  getPublisherConfigMappings,
  getTxSenderConfigMappings,
} from './publisher/config.js';
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
export type SequencerClientConfig = PublisherConfig & TxSenderConfig & SequencerConfig & L1ReaderConfig & ChainConfig;

export const sequencerConfigMappings: ConfigMappingsType<SequencerConfig> = {
  transactionPollingIntervalMS: {
    env: 'SEQ_TX_POLLING_INTERVAL_MS',
    description: 'The number of ms to wait between polling for pending txs.',
    ...numberConfigHelper(1_000),
  },
  maxTxsPerBlock: {
    env: 'SEQ_MAX_TX_PER_BLOCK',
    description: 'The maximum number of txs to include in a block.',
    ...numberConfigHelper(32),
  },
  minTxsPerBlock: {
    env: 'SEQ_MIN_TX_PER_BLOCK',
    description: 'The minimum number of txs to include in a block.',
    ...numberConfigHelper(1),
  },
  minSecondsBetweenBlocks: {
    env: 'SEQ_MIN_SECONDS_BETWEEN_BLOCKS',
    description: 'The minimum number of seconds in-between consecutive blocks.',
    ...numberConfigHelper(0),
  },
  maxSecondsBetweenBlocks: {
    env: 'SEQ_MAX_SECONDS_BETWEEN_BLOCKS',
    description:
      'The maximum number of seconds in-between consecutive blocks. Sequencer will produce a block with less than minTxsPerBlock once this threshold is reached.',
    ...numberConfigHelper(0),
  },
  coinbase: {
    env: 'COINBASE',
    parseEnv: (val: string) => EthAddress.fromString(val),
    description: 'Recipient of block reward.',
  },
  feeRecipient: {
    env: 'FEE_RECIPIENT',
    parseEnv: (val: string) => AztecAddress.fromString(val),
    description: 'Address to receive fees.',
  },
  acvmWorkingDirectory: {
    env: 'ACVM_WORKING_DIRECTORY',
    description: 'The working directory to use for simulation/proving',
  },
  acvmBinaryPath: {
    env: 'ACVM_BINARY_PATH',
    description: 'The path to the ACVM binary',
  },
  allowedInSetup: {
    env: 'SEQ_ALLOWED_SETUP_FN',
    parseEnv: (val: string) => parseSequencerAllowList(val),
    defaultValue: getDefaultAllowedSetupFunctions(),
    description: 'The list of functions calls allowed to run in setup',
    printDefault: () =>
      'AuthRegistry, FeeJuice.increase_public_balance, Token.increase_public_balance, FPC.prepare_fee',
  },
  allowedInTeardown: {
    env: 'SEQ_ALLOWED_TEARDOWN_FN',
    parseEnv: (val: string) => parseSequencerAllowList(val),
    defaultValue: getDefaultAllowedTeardownFunctions(),
    description: 'The list of functions calls allowed to run teardown',
    printDefault: () => 'FPC.pay_refund, FPC.pay_refund_with_shielded_rebate',
  },
  maxBlockSizeInBytes: {
    env: 'SEQ_MAX_BLOCK_SIZE_IN_BYTES',
    description: 'Max block size',
    ...numberConfigHelper(1024 * 1024),
  },
  enforceFees: {
    env: 'ENFORCE_FEES',
    description: 'Whether to require every tx to have a fee payer',
    ...booleanConfigHelper(),
  },
};

export const chainConfigMappings: ConfigMappingsType<ChainConfig> = {
  l1ChainId: l1ReaderConfigMappings.l1ChainId,
  version: {
    env: 'VERSION',
    description: 'The version of the rollup.',
    ...numberConfigHelper(1),
  },
};

export const sequencerClientConfigMappings: ConfigMappingsType<SequencerClientConfig> = {
  ...sequencerConfigMappings,
  ...getTxSenderConfigMappings('SEQ'),
  ...getPublisherConfigMappings('SEQ'),
  ...l1ReaderConfigMappings,
  ...chainConfigMappings,
};

/**
 * Creates an instance of SequencerClientConfig out of environment variables using sensible defaults for integration testing if not set.
 */
export function getConfigEnvVars(): SequencerClientConfig {
  return getConfigFromMappings<SequencerClientConfig>(sequencerClientConfigMappings);
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
      address: FeeJuiceAddress,
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
