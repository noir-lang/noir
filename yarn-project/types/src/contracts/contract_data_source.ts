import { type ContractArtifact, type FunctionSelector } from '@aztec/foundation/abi';
import { type AztecAddress } from '@aztec/foundation/aztec-address';
import { type Fr } from '@aztec/foundation/fields';

import { type ContractClassPublic, type PublicFunction } from './contract_class.js';
import { type ContractInstanceWithAddress } from './contract_instance.js';

export interface ContractDataSource {
  /**
   * Returns a contract's encoded public function, given its function selector.
   * @param address - The contract aztec address.
   * @param selector - The function's selector.
   * @returns The function's data.
   */
  getPublicFunction(address: AztecAddress, selector: FunctionSelector): Promise<PublicFunction | undefined>;

  /**
   * Gets the number of the latest L2 block processed by the implementation.
   * @returns The number of the latest L2 block processed by the implementation.
   */
  getBlockNumber(): Promise<number>;

  /**
   * Returns the contract class for a given contract class id, or undefined if not found.
   * @param id - Contract class id.
   */
  getContractClass(id: Fr): Promise<ContractClassPublic | undefined>;

  /**
   * Returns a publicly deployed contract instance given its address.
   * @param address - Address of the deployed contract.
   */
  getContract(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined>;

  /**
   * Returns the list of all class ids known.
   */
  getContractClassIds(): Promise<Fr[]>;

  getContractArtifact(address: AztecAddress): Promise<ContractArtifact | undefined>;
  addContractArtifact(address: AztecAddress, contract: ContractArtifact): Promise<void>;
}
