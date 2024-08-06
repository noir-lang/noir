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
  'gasTokenAddress',
  'gasPortalAddress',
] as const;

/**
 * Provides the directory of current L1 contract addresses
 */
export type L1ContractAddresses = {
  [K in (typeof l1ContractsNames)[number]]: EthAddress;
};

export function getL1ContractAddressesFromEnv() {
  const {
    AVAILABILITY_ORACLE_CONTRACT_ADDRESS,
    ROLLUP_CONTRACT_ADDRESS,
    REGISTRY_CONTRACT_ADDRESS,
    INBOX_CONTRACT_ADDRESS,
    OUTBOX_CONTRACT_ADDRESS,
    GAS_TOKEN_CONTRACT_ADDRESS,
    GAS_PORTAL_CONTRACT_ADDRESS,
  } = process.env;

  return {
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
