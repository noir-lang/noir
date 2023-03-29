import { EthAddress } from '@aztec/ethereum.js/eth_address';

export const INITIAL_L2_BLOCK_NUM = 1;

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
