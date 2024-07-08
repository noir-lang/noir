import { type AztecAddress } from '@aztec/circuits.js';
import {
  type ContractArtifact,
  type FunctionArtifact,
  type FunctionDebugMetadata,
  type FunctionSelector,
  getFunctionDebugMetadata,
} from '@aztec/foundation/abi';
import { type Fr } from '@aztec/foundation/fields';
import { ContractClassNotFoundError, ContractNotFoundError } from '@aztec/simulator';
import { type ContractClass, type ContractInstance } from '@aztec/types/contracts';

import { type ContractArtifactDatabase } from '../database/contracts/contract_artifact_db.js';
import { type ContractInstanceDatabase } from '../database/contracts/contract_instance_db.js';
import { PrivateFunctionsTree } from './private_functions_tree.js';

/**
 * ContractDataOracle serves as a data manager and retriever for Aztec.nr contracts.
 * It provides methods to obtain contract addresses, function ABI, bytecode, and membership witnesses
 * from a given contract address and function selector. The class maintains a cache of ContractTree instances
 * to efficiently serve the requested data. It interacts with the ContractDatabase and AztecNode to fetch
 * the required information and facilitate cryptographic proof generation.
 */
export class ContractDataOracle {
  /** Map from contract class id to private function tree. */
  private contractClasses: Map<string, PrivateFunctionsTree> = new Map();
  /** Map from address to contract instance. */
  private contractInstances: Map<string, ContractInstance> = new Map();

  constructor(private db: ContractArtifactDatabase & ContractInstanceDatabase) {}

  /** Returns a contract instance for a given address. Throws if not found. */
  public async getContractInstance(contractAddress: AztecAddress): Promise<ContractInstance> {
    if (!this.contractInstances.has(contractAddress.toString())) {
      const instance = await this.db.getContractInstance(contractAddress);
      if (!instance) {
        throw new ContractNotFoundError(contractAddress.toString());
      }
      this.contractInstances.set(contractAddress.toString(), instance);
    }
    return this.contractInstances.get(contractAddress.toString())!;
  }

  /** Returns a contract class for a given class id. Throws if not found. */
  public async getContractClass(contractClassId: Fr): Promise<ContractClass> {
    const tree = await this.getTreeForClassId(contractClassId);
    return tree.getContractClass();
  }

  public async getContractArtifact(contractClassId: Fr): Promise<ContractArtifact> {
    const tree = await this.getTreeForClassId(contractClassId);
    return tree.getArtifact();
  }

  /**
   * Retrieves the artifact of a specified function within a given contract.
   * The function is identified by its selector, which is a unique code generated from the function's signature.
   * Throws an error if the contract address or function selector are invalid or not found.
   *
   * @param contractAddress - The AztecAddress representing the contract containing the function.
   * @param selector - The function selector.
   * @returns The corresponding function's artifact as an object.
   */
  public async getFunctionArtifact(contractAddress: AztecAddress, selector: FunctionSelector) {
    const tree = await this.getTreeForAddress(contractAddress);
    return tree.getFunctionArtifact(selector);
  }

  /**
   * Retrieves the artifact of a specified function within a given contract.
   * The function is identified by its name, which is unique within a contract.
   * Throws if the contract has not been added to the database.
   *
   * @param contractAddress - The AztecAddress representing the contract containing the function.
   * @param functionName - The name of the function.
   * @returns The corresponding function's artifact as an object
   */
  public async getFunctionArtifactByName(
    contractAddress: AztecAddress,
    functionName: string,
  ): Promise<FunctionArtifact | undefined> {
    const tree = await this.getTreeForAddress(contractAddress);
    return tree.getArtifact().functions.find(f => f.name === functionName);
  }

  /**
   * Retrieves the debug metadata of a specified function within a given contract.
   * The function is identified by its selector, which is a unique code generated from the function's signature.
   * Returns undefined if the debug metadata for the given function is not found.
   * Throws if the contract has not been added to the database.
   *
   * @param contractAddress - The AztecAddress representing the contract containing the function.
   * @param selector - The function selector.
   * @returns The corresponding function's artifact as an object.
   */
  public async getFunctionDebugMetadata(
    contractAddress: AztecAddress,
    selector: FunctionSelector,
  ): Promise<FunctionDebugMetadata | undefined> {
    const tree = await this.getTreeForAddress(contractAddress);
    const artifact = tree.getFunctionArtifact(selector);
    return getFunctionDebugMetadata(tree.getArtifact(), artifact);
  }

  /**
   * Retrieve the bytecode of a specific function in a contract at the given address.
   * The returned bytecode is required for executing and verifying the function's behavior
   * in the Aztec network. Throws an error if the contract or function cannot be found.
   *
   * @param contractAddress - The contract's address.
   * @param selector - The function selector.
   * @returns A Promise that resolves to a Buffer containing the bytecode of the specified function.
   * @throws Error if the contract address is unknown or not found.
   */
  public async getBytecode(contractAddress: AztecAddress, selector: FunctionSelector) {
    const tree = await this.getTreeForAddress(contractAddress);
    return tree.getBytecode(selector);
  }

  /**
   * Retrieve the function membership witness for the given contract address and function selector.
   * The function membership witness represents a proof that the function belongs to the specified contract.
   * Throws an error if the contract address or function selector is unknown.
   *
   * @param contractAddress - The contract address.
   * @param selector - The function selector.
   * @returns A promise that resolves with the MembershipWitness instance for the specified contract's function.
   */
  public async getFunctionMembershipWitness(contractAddress: AztecAddress, selector: FunctionSelector) {
    const tree = await this.getTreeForAddress(contractAddress);
    return tree.getFunctionMembershipWitness(selector);
  }

  public async getDebugFunctionName(contractAddress: AztecAddress, selector: FunctionSelector) {
    const tree = await this.getTreeForAddress(contractAddress);
    const { name: contractName } = tree.getArtifact();
    const { name: functionName } = tree.getFunctionArtifact(selector);
    return `${contractName}:${functionName}`;
  }

  /**
   * Retrieve or create a ContractTree instance based on the provided class id.
   * If an existing tree with the same class id is found in the cache, it will be returned.
   * Otherwise, a new ContractTree instance will be created using the contract data from the database
   * and added to the cache before returning.
   *
   * @param classId - The class id of the contract for which the ContractTree is required.
   * @returns A ContractTree instance associated with the specified contract address.
   * @throws An Error if the contract is not found in the ContractDatabase.
   */
  private async getTreeForClassId(classId: Fr): Promise<PrivateFunctionsTree> {
    if (!this.contractClasses.has(classId.toString())) {
      const artifact = await this.db.getContractArtifact(classId);
      if (!artifact) {
        throw new ContractClassNotFoundError(classId.toString());
      }
      const tree = new PrivateFunctionsTree(artifact);
      this.contractClasses.set(classId.toString(), tree);
    }
    return this.contractClasses.get(classId.toString())!;
  }

  /**
   * Retrieve or create a ContractTree instance based on the provided AztecAddress.
   * If an existing tree with the same contract address is found in the cache, it will be returned.
   * Otherwise, a new ContractTree instance will be created using the contract data from the database
   * and added to the cache before returning.
   *
   * @param contractAddress - The AztecAddress of the contract for which the ContractTree is required.
   * @returns A ContractTree instance associated with the specified contract address.
   * @throws An Error if the contract is not found in the ContractDatabase.
   */
  private async getTreeForAddress(contractAddress: AztecAddress): Promise<PrivateFunctionsTree> {
    const instance = await this.getContractInstance(contractAddress);
    return this.getTreeForClassId(instance.contractClassId);
  }
}
