import { AztecNode } from '@aztec/aztec-node';
import {
  CONTRACT_TREE_HEIGHT,
  FUNCTION_TREE_HEIGHT,
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
import { CircuitsWasm } from '@aztec/circuits.js/wasm';
import { AztecAddress, EthAddress, Fr, keccak } from '@aztec/foundation';
import { ContractAbi, FunctionType } from '@aztec/noir-contracts';
import { generateFunctionSelector } from '../abi_coder/index.js';
import { ContractDao, ContractFunctionDao } from '../contract_database/index.js';
import { computeFunctionTreeData } from './function_tree_data.js';

async function hashVKStr(vk: string, wasm: CircuitsWasm) {
  // TODO - check consistent encoding
  return await hashVK(wasm, Buffer.from(vk, 'hex'));
}

function isConstructor({ name }: { name: string }) {
  return name === 'constructor';
}

/**
 * @param functions - Function name and type.
 * @returns Boolean indicating if the function is constrained and therefore in the function tree.
 */
function isConstrained({ name, functionType }: { name: string; functionType: FunctionType }) {
  return functionType !== FunctionType.UNCONSTRAINED && !isConstructor({ name });
}

async function generateFunctionLeaves(functions: ContractFunctionDao[], wasm: CircuitsWasm) {
  const targetFunctions = functions.filter(isConstrained);
  const result: Fr[] = [];
  for (let i = 0; i < targetFunctions.length; i++) {
    const f = targetFunctions[i];
    const selector = generateFunctionSelector(f.name, f.parameters);
    const isPrivate = f.functionType === FunctionType.SECRET;
    // All non-unconstrained functions have vks
    const vkHash = await hashVKStr(f.verificationKey!, wasm);
    const acirHash = keccak(Buffer.from(f.bytecode, 'hex'));
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

export interface NewContractConstructor {
  functionData: FunctionData;
  vkHash: Buffer;
}

export class ContractTree {
  private functionLeaves?: Fr[];
  private functionTree?: Fr[];
  private functionTreeRoot?: Fr;
  private contractMembershipWitness?: MembershipWitness<typeof CONTRACT_TREE_HEIGHT>;

  constructor(
    public readonly contract: ContractDao,
    private node: AztecNode,
    private wasm: CircuitsWasm,
    public readonly newContractConstructor?: NewContractConstructor,
  ) {}

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

  public getFunctionAbi(functionSelector: Buffer) {
    const abi = this.contract.functions.find(f => f.selector.equals(functionSelector));
    if (!abi) {
      throw new Error(`Unknown function: ${functionSelector}.`);
    }
    return abi;
  }

  public getBytecode(functionSelector: Buffer) {
    return this.getFunctionAbi(functionSelector).bytecode;
  }

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
        Number(index),
        siblingPath.data.map(x => new Fr(x.readBigInt64BE())),
      );
    }
    return this.contractMembershipWitness;
  }

  public async getFunctionTreeRoot() {
    if (!this.functionTreeRoot) {
      const leaves = await this.getFunctionLeaves();
      this.functionTreeRoot = await computeFunctionTreeRoot(this.wasm, leaves);
    }
    return this.functionTreeRoot;
  }

  public async getFunctionMembershipWitness(functionSelector: Buffer) {
    const targetFunctions = this.contract.functions.filter(isConstrained);
    const functionIndex = targetFunctions.findIndex(f => f.selector.equals(functionSelector));
    if (functionIndex < 0) {
      return MembershipWitness.empty(FUNCTION_TREE_HEIGHT, 0);
    }

    if (!this.functionTree) {
      const leaves = await this.getFunctionLeaves();
      this.functionTree = await computeFunctionTree(this.wasm, leaves);
    }
    const functionTreeData = computeFunctionTreeData(this.functionTree, functionIndex);
    return new MembershipWitness<typeof FUNCTION_TREE_HEIGHT>(
      FUNCTION_TREE_HEIGHT,
      functionIndex,
      functionTreeData.siblingPath,
    );
  }

  private async getFunctionLeaves() {
    if (!this.functionLeaves) {
      this.functionLeaves = await generateFunctionLeaves(this.contract.functions, this.wasm);
    }
    return this.functionLeaves;
  }
}
