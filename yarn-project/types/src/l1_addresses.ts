import { EthAddress } from '@aztec/foundation/eth-address';

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
