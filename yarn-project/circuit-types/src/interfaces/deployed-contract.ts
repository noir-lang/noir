import { ContractArtifact } from '@aztec/foundation/abi';
import { ContractInstanceWithAddress } from '@aztec/types/contracts';

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
   * The contract instance.
   */
  instance: ContractInstanceWithAddress;
}
