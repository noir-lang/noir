import { EthAddress } from '@aztec/circuits.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { CompleteAddress } from '@aztec/types';

/**
 * Represents a deployed contract on the Aztec network.
 * Contains the contract ABI, address, and associated portal contract address.
 */
export interface DeployedContract {
  /**
   * The Application Binary Interface of the deployed contract.
   */
  abi: ContractAbi;
  /**
   * The complete address representing the contract on L2.
   */
  completeAddress: CompleteAddress;
  /**
   * The Ethereum address of the L1 portal contract.
   */
  portalContract: EthAddress;
}
