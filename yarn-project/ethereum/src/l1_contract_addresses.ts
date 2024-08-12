import { type ConfigMappingsType, getConfigFromMappings } from '@aztec/foundation/config';
import { EthAddress } from '@aztec/foundation/eth-address';
import type { DebugLogger } from '@aztec/foundation/log';

/**
 * The names of the current L1 contract addresses.
 * NOTE: When changing this list, make sure to update CLI & CI scripts accordingly.
 * For reference: https://github.com/AztecProtocol/aztec-packages/pull/5553
 */
export const l1ContractsNames = [
  'availabilityOracleAddress',
  'rollupAddress',
  'registryAddress',
  'inboxAddress',
  'outboxAddress',
  'feeJuiceAddress',
  'feeJuicePortalAddress',
] as const;

/**
 * Provides the directory of current L1 contract addresses
 */
export type L1ContractAddresses = {
  [K in (typeof l1ContractsNames)[number]]: EthAddress;
};

const parseEnv = (val: string) => EthAddress.fromString(val);

export const l1ContractAddressesMapping: ConfigMappingsType<L1ContractAddresses> = {
  availabilityOracleAddress: {
    env: 'AVAILABILITY_ORACLE_CONTRACT_ADDRESS',
    description: 'The deployed L1 availability oracle contract address.',
    parseEnv,
  },
  rollupAddress: {
    env: 'ROLLUP_CONTRACT_ADDRESS',
    description: 'The deployed L1 rollup contract address.',
    parseEnv,
  },
  registryAddress: {
    env: 'REGISTRY_CONTRACT_ADDRESS',
    description: 'The deployed L1 registry contract address.',
    parseEnv,
  },
  inboxAddress: {
    env: 'INBOX_CONTRACT_ADDRESS',
    description: 'The deployed L1 inbox contract address.',
    parseEnv,
  },
  outboxAddress: {
    env: 'OUTBOX_CONTRACT_ADDRESS',
    description: 'The deployed L1 outbox contract address.',
    parseEnv,
  },
  feeJuiceAddress: {
    env: 'FEE_JUICE_CONTRACT_ADDRESS',
    description: 'The deployed L1 Fee Juice contract address.',
    parseEnv,
  },
  feeJuicePortalAddress: {
    env: 'FEE_JUICE_PORTAL_CONTRACT_ADDRESS',
    description: 'The deployed L1 Fee Juice portal contract address.',
    parseEnv,
  },
};

export function getL1ContractAddressesFromEnv() {
  return getConfigFromMappings<L1ContractAddresses>(l1ContractAddressesMapping);
}

function convertToL1ContractAddresses(obj: any): L1ContractAddresses {
  if (typeof obj !== 'object' || obj === null) {
    throw new Error('Object is not valid');
  }

  const result: Partial<L1ContractAddresses> = {};

  for (const key of l1ContractsNames) {
    const value = obj[key];
    result[key] = EthAddress.fromString(value);
  }

  return result as L1ContractAddresses;
}

export async function getL1ContractAddressesFromUrl(url: string, log: DebugLogger): Promise<L1ContractAddresses> {
  try {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP error when fetching L1 contracts from ${url}. Status: ${response.status}`);
    }
    const data = await response.json();
    return convertToL1ContractAddresses(data);
  } catch (error) {
    log.error(`Error fetching JSON from ${url}:`, error);
    throw error;
  }
}
