import { EthAddress } from '@aztec/circuits.js';

/**
 * Provides basic information about the running node.
 */
export type NodeInfo = {
  /**
   * Version as tracked in the aztec-packages repository.
   */
  sandboxVersion: string;
  /**
   * The nargo version compatible with this sandbox version
   */
  compatibleNargoVersion: string;
  /**
   * L1 chain id.
   */
  chainId: number;
  /**
   * Protocol version.
   */
  protocolVersion: number;
  /**
   * The rollup contract address
   */
  rollupAddress: EthAddress;
};
