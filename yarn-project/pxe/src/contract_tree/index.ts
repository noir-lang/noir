import { ContractDao, MerkleTreeId, StateInfoProvider } from '@aztec/circuit-types';
import {
  CONTRACT_TREE_HEIGHT,
  FUNCTION_TREE_HEIGHT,
  Fr,
  MembershipWitness,
  NewContractConstructor,
  NewContractData,
  computeFunctionTreeData,
  generateFunctionLeaves,
  getContractClassFromArtifact,
  isConstrained,
} from '@aztec/circuits.js';
import { computeContractLeaf, computeFunctionTree, computeFunctionTreeRoot } from '@aztec/circuits.js/abis';
import { FunctionSelector } from '@aztec/foundation/abi';
import { assertLength } from '@aztec/foundation/serialize';

/**
 * The ContractTree class represents a Merkle tree of functions for a particular contract.
 * It manages the construction of the function tree, computes its root, and generates membership witnesses
 * for constrained functions. This class also enables lookup of specific function artifact using selectors.
 * It is used in combination with the AztecNode to compute various data for executing private transactions.
 */
export class ContractTree {
  private functionLeaves?: Fr[];
  private functionTree?: Fr[];
  private functionTreeRoot?: Fr;
  private contractIndex?: bigint;
  private contractClassId?: Fr;

  constructor(
    /**
     * The contract data object containing the artifact and contract address.
     */
    public readonly contract: ContractDao,
    private stateInfoProvider: StateInfoProvider,
    /**
     * Data associated with the contract constructor for a new contract.
     */
    public readonly newContractConstructor?: NewContractConstructor,
  ) {}

  /**
   * Retrieve the artifact of a given function.
   * The function is identified by its selector, which represents a unique identifier for the function's signature.
   * Throws an error if the function with the provided selector is not found in the contract.
   *
   * @param selector - The function selector.
   * @returns The artifact object containing relevant information about the targeted function.
   */
  public getFunctionArtifact(selector: FunctionSelector) {
    const artifact = this.contract.functions.find(f => f.selector.equals(selector));
    if (!artifact) {
      throw new Error(
        `Unknown function. Selector ${selector.toString()} not found in the artifact of contract ${this.contract.instance.address.toString()}. Expected one of: ${this.contract.functions
          .map(f => f.selector.toString())
          .join(', ')}`,
      );
    }
    return artifact;
  }

  /**
   * Retrieve the bytecode of a function in the contract by its function selector.
   * The function selector is a unique identifier for each function in a contract.
   * Throws an error if the function with the given selector is not found in the contract.
   *
   * @param selector - The selector of a function to get bytecode for.
   * @returns The bytecode of the function as a string.
   */
  public getBytecode(selector: FunctionSelector) {
    return this.getFunctionArtifact(selector).bytecode;
  }

  /**
   * Retrieves the contract membership witness for the current contract tree instance.
   * The contract membership witness is a proof that demonstrates the existence of the contract
   * in the global contract merkle tree. This proof contains the index of the contract's leaf
   * in the tree and the sibling path needed to construct the root of the merkle tree.
   * If the witness hasn't been previously computed, this function will request the contract node
   * to find the contract's index and path in order to create the membership witness.
   *
   * @param blockNumber - The block number at which to get the data.
   *
   * @returns A Promise that resolves to the MembershipWitness object for the given contract tree.
   */
  public async getContractMembershipWitness(blockNumber: number | 'latest' = 'latest') {
    const index = await this.getContractIndex();

    const siblingPath = await this.stateInfoProvider.getContractSiblingPath(blockNumber, index);
    return new MembershipWitness<typeof CONTRACT_TREE_HEIGHT>(
      CONTRACT_TREE_HEIGHT,
      index,
      assertLength(siblingPath.toFieldArray(), CONTRACT_TREE_HEIGHT),
    );
  }

  /**
   * Calculate and return the root of the function tree for the current contract.
   * This root is a cryptographic commitment to the set of constrained functions within the contract,
   * which is used in the Aztec node's proof system. The root will be cached after the first call.
   *
   * @returns A promise that resolves to the Fr (finite field element) representation of the function tree root.
   */
  public getFunctionTreeRoot() {
    if (!this.functionTreeRoot) {
      const leaves = this.getFunctionLeaves();
      this.functionTreeRoot = computeFunctionTreeRoot(leaves);
    }
    return Promise.resolve(this.functionTreeRoot);
  }

  /**
   * Returns the contract class identifier for the given artifact.
   */
  public getContractClassId() {
    if (!this.contractClassId) {
      this.contractClassId = getContractClassFromArtifact(this.contract).id;
    }
    return this.contractClassId;
  }

  /**
   * Retrieve the membership witness of a function within a contract's function tree.
   * A membership witness represents the position and authentication path of a target function
   * in the Merkle tree of constrained functions. It is required to prove the existence of the
   * function within the contract during execution.
   *
   * @param selector - The function selector.
   * @returns A MembershipWitness instance representing the position and authentication path of the function in the function tree.
   */
  public getFunctionMembershipWitness(
    selector: FunctionSelector,
  ): Promise<MembershipWitness<typeof FUNCTION_TREE_HEIGHT>> {
    const targetFunctions = this.contract.functions.filter(isConstrained);
    const functionIndex = targetFunctions.findIndex(f => f.selector.equals(selector));
    if (functionIndex < 0) {
      return Promise.resolve(MembershipWitness.empty(FUNCTION_TREE_HEIGHT, 0n));
    }

    if (!this.functionTree) {
      const leaves = this.getFunctionLeaves();
      this.functionTree = computeFunctionTree(leaves);
    }
    const functionTreeData = computeFunctionTreeData(this.functionTree, functionIndex);
    return Promise.resolve(
      new MembershipWitness<typeof FUNCTION_TREE_HEIGHT>(
        FUNCTION_TREE_HEIGHT,
        BigInt(functionIndex),
        assertLength(functionTreeData.siblingPath, FUNCTION_TREE_HEIGHT),
      ),
    );
  }

  /**
   * Retrieve the function leaves for the contract tree.
   * Function leaves are computed based on constrained functions present in the contract.
   * It caches the computed function leaves and returns them if already calculated.
   *
   * @returns An array of Fr representing the function leaves.
   */
  private getFunctionLeaves() {
    if (!this.functionLeaves) {
      this.functionLeaves = generateFunctionLeaves(this.contract.functions);
    }
    return this.functionLeaves;
  }

  private async getContractIndex() {
    if (this.contractIndex === undefined) {
      const { address, portalContractAddress } = this.contract.instance;
      const contractClassId = this.getContractClassId();
      const newContractData = new NewContractData(address, portalContractAddress, contractClassId);
      const commitment = computeContractLeaf(newContractData);
      this.contractIndex = await this.stateInfoProvider.findLeafIndex('latest', MerkleTreeId.CONTRACT_TREE, commitment);
      if (this.contractIndex === undefined) {
        throw new Error(`Failed to find contract at ${address.toString()} resulting in commitment ${commitment}.`);
      }
      return this.contractIndex;
    }
    return this.contractIndex;
  }
}
