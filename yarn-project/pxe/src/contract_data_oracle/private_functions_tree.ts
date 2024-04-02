import {
  FUNCTION_TREE_HEIGHT,
  MembershipWitness,
  computePrivateFunctionLeaf,
  computePrivateFunctionsTree,
  getContractClassFromArtifact,
} from '@aztec/circuits.js';
import { type MerkleTree } from '@aztec/circuits.js/merkle';
import { type ContractArtifact, type FunctionSelector } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { assertLength } from '@aztec/foundation/serialize';
import { type ContractClassWithId } from '@aztec/types/contracts';

/**
 * Represents a Merkle tree of functions for a particular Contract Class.
 * It manages the construction of the function tree, computes its root, and generates membership witnesses
 * for constrained functions. This class also enables lookup of specific function artifact using selectors.
 * It is used in combination with the AztecNode to compute various data for executing private transactions.
 */
export class PrivateFunctionsTree {
  private tree?: MerkleTree;
  private contractClass: ContractClassWithId;

  constructor(private readonly artifact: ContractArtifact) {
    this.contractClass = getContractClassFromArtifact(artifact);
  }

  /**
   * Retrieve the artifact of a given function.
   * The function is identified by its selector, which represents a unique identifier for the function's signature.
   * Throws an error if the function with the provided selector is not found in the contract.
   *
   * @param selector - The function selector.
   * @returns The artifact object containing relevant information about the targeted function.
   */
  public getFunctionArtifact(selector: FunctionSelector) {
    const artifact = this.artifact.functions.find(f => selector.equals(f.name, f.parameters));
    if (!artifact) {
      throw new Error(
        `Unknown function. Selector ${selector.toString()} not found in the artifact with class ${this.getContractClassId().toString()}.`,
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
   * Calculate and return the root of the function tree for the current contract.
   * This root is a cryptographic commitment to the set of constrained functions within the contract,
   * which is used in the Aztec node's proof system. The root will be cached after the first call.
   *
   * @returns A promise that resolves to the Fr (finite field element) representation of the function tree root.
   */
  public getFunctionTreeRoot() {
    return this.getTree();
  }

  /** Returns the contract class object. */
  public getContractClass() {
    return this.contractClass;
  }

  /** Returns the contract artifact. */
  public getArtifact() {
    return this.artifact;
  }

  /**
   * Returns the contract class identifier for the given artifact.
   */
  public getContractClassId() {
    return this.getContractClass().id;
  }

  /**
   * Retrieve the membership witness of a function within a contract's function tree.
   * A membership witness represents the position and authentication path of a target function
   * in the Merkle tree of constrained functions. It is required to prove the existence of the
   * function within the contract during execution. Throws if fn does not exist or is not private.
   *
   * @param selector - The function selector.
   * @returns A MembershipWitness instance representing the position and authentication path of the function in the function tree.
   */
  public getFunctionMembershipWitness(
    selector: FunctionSelector,
  ): Promise<MembershipWitness<typeof FUNCTION_TREE_HEIGHT>> {
    const fn = this.getContractClass().privateFunctions.find(f => f.selector.equals(selector));
    if (!fn) {
      throw new Error(`Private function with selector ${selector.toString()} not found in contract class.`);
    }

    const leaf = computePrivateFunctionLeaf(fn);
    const index = this.getTree().getIndex(leaf);
    const path = this.getTree().getSiblingPath(index);
    return Promise.resolve(
      new MembershipWitness<typeof FUNCTION_TREE_HEIGHT>(
        FUNCTION_TREE_HEIGHT,
        BigInt(index),
        assertLength(path.map(Fr.fromBuffer), FUNCTION_TREE_HEIGHT),
      ),
    );
  }

  private getTree() {
    if (!this.tree) {
      const fns = this.getContractClass().privateFunctions;
      this.tree = computePrivateFunctionsTree(fns);
    }
    return this.tree;
  }
}
