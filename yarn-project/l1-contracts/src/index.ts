import { EthAddress } from '@aztec/foundation/eth-address';

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
   * UnverifiedDataEmitter contract address.
   */
  unverifiedDataEmitterContract: EthAddress;
}

export * from './ethereumjs-contracts/DecoderHelper.js';
export * from './ethereumjs-contracts/Rollup.js';
export * from './ethereumjs-contracts/UnverifiedDataEmitter.js';
