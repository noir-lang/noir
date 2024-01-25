import { ContractArtifact } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';

/**
 * PXE database for managing contract artifacts.
 */
export interface ContractArtifactDatabase {
  /**
   * Adds a new contract artifact to the database or updates an existing one.
   * @param id - Id of the corresponding contract class.
   * @param contract - Contract artifact to add.
   */
  addContractArtifact(id: Fr, contract: ContractArtifact): Promise<void>;
  /**
   * Gets a contract artifact given its resulting contract class id.
   * @param id - Contract class id for the given artifact.
   */
  getContractArtifact(id: Fr): Promise<ContractArtifact | undefined>;
}
