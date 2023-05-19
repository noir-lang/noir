import { AztecNode } from '@aztec/aztec-node';
import {
  AztecAddress,
  CONTRACT_TREE_HEIGHT,
  CircuitsWasm,
  EthAddress,
  FUNCTION_TREE_HEIGHT,
  Fr,
  FunctionData,
  FunctionLeafPreimage,
  MembershipWitness,
  NewContractData,
  computeFunctionTree,
} from '@aztec/circuits.js';
import {
  computeContractAddress,
  computeContractLeaf,
  computeFunctionLeaf,
  computeFunctionTreeRoot,
  hashConstructor,
  hashVK,
} from '@aztec/circuits.js/abis';
import { FunctionType, ContractAbi } from '@aztec/foundation/abi';
import { assertLength } from '@aztec/foundation/serialize';
import { ContractFunctionDao, ContractDao } from '../contract_database/contract_dao.js';
import { generateFunctionSelector } from '../index.js';
import { computeFunctionTreeData } from './function_tree_data.js';

/**
 * Computes the hash of a hex-encoded string representation of a verification key (vk).
 * The input 'vk' should be a hexadecimal string, and the resulting hash is computed using 'hashVK' function.
 * Returns a Promise that resolves to a Buffer containing the hash of the verification key.
 *
 * @param vk - The hex-encoded string representing the verification key.
 * @param wasm - An instance of CircuitsWasm class used for hashing.
 * @returns A Promise resolving to a Buffer containing the hash of the verification key.
 */
async function hashVKStr(vk: string, wasm: CircuitsWasm) {
  // TODO - check consistent encoding
  return await hashVK(wasm, Buffer.from(vk, 'hex'));
}

/**
 * Determine if the given function is a constructor.
 * This utility function checks if the 'name' property of the input object is "constructor".
 * Returns true if the function is a constructor, false otherwise.
 *
 * @param Object - An object containing a 'name' property.
 * @returns Boolean indicating if the function is a constructor.
 */
function isConstructor({
  name,
}: {
  /**
   * Function name identifier.
   */
  name: string;
}) {
  return name === 'constructor';
}

/**
 * @param Object - An object containing function name and type.
 * @returns Boolean indicating if the function is constrained and therefore in the function tree.
 */
function isConstrained({
  name,
  functionType,
}: {
  /**
   * The name of the contract function.
   */
  name: string;
  /**
   * The type of a contract function determining its constraints.
   */
  functionType: FunctionType;
}) {
  return functionType !== FunctionType.UNCONSTRAINED && !isConstructor({ name });
}

/**
 * Generate function leaves for the constrained functions in a contract.
 * Only computes leaves for functions that are either secret or open and not constructors.
 * Each function leaf is computed from its selector, privacy flag, hashed verification key, and hashed bytecode.
 *
 * @param functions - Array of ContractFunctionDao objects representing the functions in a contract.
 * @param wasm - CircuitsWasm instance used for hashing and computations.
 * @returns An array of Fr instances representing the generated function leaves.
 */
async function generateFunctionLeaves(functions: ContractFunctionDao[], wasm: CircuitsWasm) {
  const targetFunctions = functions.filter(isConstrained);
  const result: Fr[] = [];
  for (let i = 0; i < targetFunctions.length; i++) {
    const f = targetFunctions[i];
    const selector = generateFunctionSelector(f.name, f.parameters);
    const isPrivate = f.functionType === FunctionType.SECRET;
    // All non-unconstrained functions have vks
    const vkHash = await hashVKStr(f.verificationKey!, wasm);
    // TODO
    // FIXME: https://github.com/AztecProtocol/aztec3-packages/issues/262
    // const acirHash = keccak(Buffer.from(f.bytecode, 'hex'));
    const acirHash = Buffer.alloc(32, 0);

    const fnLeafPreimage = new FunctionLeafPreimage(
      selector,
      isPrivate,
      Fr.fromBuffer(vkHash),
      Fr.fromBuffer(acirHash),
    );
    const fnLeaf = await computeFunctionLeaf(wasm, fnLeafPreimage);
    result.push(fnLeaf);
  }
  return result;
}

/**
 * Represents the constructor data for a new contract.
 * Contains the function data and verification key hash required for contract creation.
 */
export interface NewContractConstructor {
  /**
   * Stores essential information about a contract function.
   */
  functionData: FunctionData;
  /**
   * The hashed verification key of a function.
   */
  vkHash: Buffer;
}

/**
 * The ContractTree class represents a Merkle tree of functions for a particular contract.
 * It manages the construction of the function tree, computes its root, and generates membership witnesses
 * for constrained functions. This class also enables lookup of specific function ABI and bytecode using selectors.
 * It is used in combination with the AztecNode to compute various data for executing private transactions.
 */
export class ContractTree {
  private functionLeaves?: Fr[];
  private functionTree?: Fr[];
  private functionTreeRoot?: Fr;
  private contractMembershipWitness?: MembershipWitness<typeof CONTRACT_TREE_HEIGHT>;

  constructor(
    /**
     * The contract data object containing the ABI and contract address.
     */
    public readonly contract: ContractDao,
    private node: AztecNode,
    private wasm: CircuitsWasm,
    /**
     * Data associated with the contract constructor for a new contract.
     */
    public readonly newContractConstructor?: NewContractConstructor,
  ) {}

  /**
   * Create a new ContractTree instance from the provided contract ABI, constructor arguments, and related data.
   * The function generates function leaves for constrained functions, computes the function tree root,
   * and hashes the constructor's verification key. It then computes the contract address using the contract
   * and portal contract addresses, contract address salt, and generated data. Finally, it returns a new
   * ContractTree instance containing the contract data and computed values.
   *
   * @param abi - The contract's ABI containing the functions and their metadata.
   * @param args - An array of Fr elements representing the constructor's arguments.
   * @param portalContract - The Ethereum address of the portal smart contract.
   * @param contractAddressSalt - An Fr element representing the salt used to compute the contract address.
   * @param from - The Aztec address of the contract deployer.
   * @param node - An instance of the AztecNode class representing the current node.
   * @returns A new ContractTree instance containing the contract data and computed values.
   */
  public static async new(
    abi: ContractAbi,
    args: Fr[],
    portalContract: EthAddress,
    contractAddressSalt: Fr,
    from: AztecAddress,
    node: AztecNode,
  ) {
    const wasm = await CircuitsWasm.get();
    const constructorAbi = abi.functions.find(isConstructor);
    if (!constructorAbi) {
      throw new Error('Constructor not found.');
    }
    if (!constructorAbi.verificationKey) {
      throw new Error('Missing verification key for the constructor.');
    }

    const functions = abi.functions.map(f => ({
      ...f,
      selector: generateFunctionSelector(f.name, f.parameters),
    }));
    const leaves = await generateFunctionLeaves(functions, wasm);
    const root = await computeFunctionTreeRoot(wasm, leaves);
    const constructorSelector = generateFunctionSelector(constructorAbi.name, constructorAbi.parameters);
    const functionData = new FunctionData(constructorSelector, true, true);
    const vkHash = await hashVKStr(constructorAbi.verificationKey, wasm);
    const constructorHash = await hashConstructor(wasm, functionData, args, vkHash);
    const address = await computeContractAddress(wasm, from, contractAddressSalt, root, constructorHash);
    const contractDao: ContractDao = {
      ...abi,
      address,
      functions,
      portalContract,
    };
    const NewContractConstructor = {
      functionData,
      vkHash,
    };
    return new ContractTree(contractDao, node, wasm, NewContractConstructor);
  }

  /**
   * Retrieve the ABI of a given function.
   * The function is identified by its selector, which represents a unique identifier for the function's signature.
   * Throws an error if the function with the provided selector is not found in the contract.
   *
   * @param functionSelector - The Buffer containing the unique identifier for the function's signature.
   * @returns The ABI object containing relevant information about the targeted function.
   */
  public getFunctionAbi(functionSelector: Buffer) {
    const abi = this.contract.functions.find(f => f.selector.equals(functionSelector));
    if (!abi) {
      throw new Error(`Unknown function: ${functionSelector}.`);
    }
    return abi;
  }

  /**
   * Retrieve the bytecode of a function in the contract by its function selector.
   * The function selector is a unique identifier for each function in a contract.
   * Throws an error if the function with the given selector is not found in the contract.
   *
   * @param functionSelector - The Buffer representing the function selector.
   * @returns The bytecode of the function as a string.
   */
  public getBytecode(functionSelector: Buffer) {
    return this.getFunctionAbi(functionSelector).bytecode;
  }

  /**
   * Retrieves the contract membership witness for the current contract tree instance.
   * The contract membership witness is a proof that demonstrates the existence of the contract
   * in the global contract merkle tree. This proof contains the index of the contract's leaf
   * in the tree and the sibling path needed to construct the root of the merkle tree.
   * If the witness hasn't been previously computed, this function will request the contract node
   * to find the contract's index and path in order to create the membership witness.
   *
   * @returns A Promise that resolves to the MembershipWitness object for the given contract tree.
   */
  public async getContractMembershipWitness() {
    if (!this.contractMembershipWitness) {
      const { address, portalContract } = this.contract;
      const root = await this.getFunctionTreeRoot();
      const newContractData = new NewContractData(address, portalContract, root);
      const committment = computeContractLeaf(this.wasm, newContractData);
      const index = await this.node.findContractIndex(committment.toBuffer());
      if (index === undefined) {
        throw new Error('Failed to find contract.');
      }

      const siblingPath = await this.node.getContractPath(index);
      this.contractMembershipWitness = new MembershipWitness<typeof CONTRACT_TREE_HEIGHT>(
        CONTRACT_TREE_HEIGHT,
        index,
        assertLength(
          siblingPath.data.map(x => Fr.fromBuffer(x)),
          CONTRACT_TREE_HEIGHT,
        ),
      );
    }
    return this.contractMembershipWitness;
  }

  /**
   * Calculate and return the root of the function tree for the current contract.
   * This root is a cryptographic commitment to the set of constrained functions within the contract,
   * which is used in the Aztec node's proof system. The root will be cached after the first call.
   *
   * @returns A promise that resolves to the Fr (finite field element) representation of the function tree root.
   */
  public async getFunctionTreeRoot() {
    if (!this.functionTreeRoot) {
      const leaves = await this.getFunctionLeaves();
      this.functionTreeRoot = await computeFunctionTreeRoot(this.wasm, leaves);
    }
    return this.functionTreeRoot;
  }

  /**
   * Retrieve the membership witness of a function within a contract's function tree.
   * A membership witness represents the position and authentication path of a target function
   * in the Merkle tree of constrained functions. It is required to prove the existence of the
   * function within the contract during execution.
   *
   * @param functionSelector - The Buffer containing the function selector (signature).
   * @returns A MembershipWitness instance representing the position and authentication path of the function in the function tree.
   */
  public async getFunctionMembershipWitness(functionSelector: Buffer) {
    const targetFunctions = this.contract.functions.filter(isConstrained);
    const functionIndex = targetFunctions.findIndex(f => f.selector.equals(functionSelector));
    if (functionIndex < 0) {
      return MembershipWitness.empty(FUNCTION_TREE_HEIGHT, 0n);
    }

    if (!this.functionTree) {
      const leaves = await this.getFunctionLeaves();
      this.functionTree = await computeFunctionTree(this.wasm, leaves);
    }
    const functionTreeData = computeFunctionTreeData(this.functionTree, functionIndex);
    return new MembershipWitness<typeof FUNCTION_TREE_HEIGHT>(
      FUNCTION_TREE_HEIGHT,
      BigInt(functionIndex),
      assertLength(functionTreeData.siblingPath, FUNCTION_TREE_HEIGHT),
    );
  }

  /**
   * Retrieve the function leaves for the contract tree.
   * Function leaves are computed based on constrained functions present in the contract.
   * It caches the computed function leaves and returns them if already calculated.
   *
   * @returns An array of Fr representing the function leaves.
   */
  private async getFunctionLeaves() {
    if (!this.functionLeaves) {
      this.functionLeaves = await generateFunctionLeaves(this.contract.functions, this.wasm);
    }
    return this.functionLeaves;
  }
}
