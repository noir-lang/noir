import { type ConfigMappingsType } from '@aztec/foundation/config';

import { type L1ContractAddresses, l1ContractAddressesMapping } from './l1_contract_addresses.js';

/**
 * Configuration of the L1GlobalReader.
 */
export interface L1ReaderConfig {
  /**
   * The RPC Url of the ethereum host.
   */
  l1RpcUrl: string;
  /**
   * The chain ID of the ethereum host.
   */
  l1ChainId: number;

  /**
   * The deployed l1 contract addresses
   */
  l1Contracts: L1ContractAddresses;
}

export const l1ReaderConfigMappings: ConfigMappingsType<L1ReaderConfig> = {
  l1RpcUrl: {
    env: 'ETHEREUM_HOST',
    description: 'The RPC Url of the ethereum host.',
  },
  l1ChainId: {
    env: 'L1_CHAIN_ID',
    parseEnv: (val: string) => +val,
    defaultValue: 31337,
    description: 'The chain ID of the ethereum host.',
  },
  // NOTE: Special case for l1Contracts
  l1Contracts: {
    description: 'The deployed L1 contract addresses',
    defaultValue: l1ContractAddressesMapping,
  },
};
