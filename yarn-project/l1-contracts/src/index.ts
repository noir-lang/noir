import { EthAddress } from '@aztec/ethereum.js/eth_address';

export const INITIAL_ROLLUP_ID = 1;

/**
 * Rollup contract addresses.
 */
export interface L1Addresses {
  /**
   * Rollup contract address.
   */
  rollupContract: EthAddress;

  /**
   * Yeeter contract address.
   */
  yeeterContract: EthAddress;
}

export * from './ethereumjs-contracts/Rollup.js';
export * from './ethereumjs-contracts/Yeeter.js';
