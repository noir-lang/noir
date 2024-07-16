import { type L1ContractAddresses } from '@aztec/ethereum';

import { type ProtocolContractAddresses } from '../contracts/protocol_contract_addresses.js';

/**
 * Provides basic information about the running node.
 */
export interface NodeInfo {
  /**
   * Version as tracked in the aztec-packages repository.
   */
  nodeVersion: string;
  /**
   * L1 chain id.
   */
  l1ChainId: number;
  /**
   * Protocol version.
   */
  protocolVersion: number;
  /**
   * The deployed l1 contract addresses
   */
  l1ContractAddresses: L1ContractAddresses;
  /**
   * Protocol contract addresses
   */
  protocolContractAddresses: ProtocolContractAddresses;
}
