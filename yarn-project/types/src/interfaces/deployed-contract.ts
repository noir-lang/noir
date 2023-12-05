import { CompleteAddress } from '@aztec/circuits.js';
import { ContractArtifact } from '@aztec/foundation/abi';
import { EthAddress } from '@aztec/foundation/eth-address';

/**
 * Represents a deployed contract on the Aztec network.
 * Contains the contract artifact, address, and associated portal contract address.
 */
export interface DeployedContract {
  /**
   * The artifact of the deployed contract.
   */
  artifact: ContractArtifact;
  /**
   * The complete address representing the contract on L2.
   */
  completeAddress: CompleteAddress;
  /**
   * The Ethereum address of the L1 portal contract.
   */
  portalContract: EthAddress;
}
