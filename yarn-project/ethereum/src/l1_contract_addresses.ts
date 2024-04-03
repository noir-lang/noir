import { type EthAddress } from '@aztec/foundation/eth-address';

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
