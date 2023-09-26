import { L1ContractAddresses } from '@aztec/ethereum';

/**
 * Provides basic information about the running node.
 */
export interface NodeInfo {
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
   * The deployed l1 contract addresses
   */
  l1ContractAddresses: L1ContractAddresses;
}
