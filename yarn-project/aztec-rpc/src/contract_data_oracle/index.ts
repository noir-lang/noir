import { AztecNode } from '@aztec/aztec-node';
import { AztecAddress, CircuitsWasm, MembershipWitness, VK_TREE_HEIGHT, VerificationKey } from '@aztec/circuits.js';
import { ContractDatabase } from '../contract_database/index.js';
import { ContractTree } from '../contract_tree/index.js';

export class ContractDataOracle {
  private trees: ContractTree[] = [];

  constructor(private db: ContractDatabase, private node: AztecNode) {}

  public async getPortalContractAddress(contractAddress: AztecAddress) {
    const tree = await this.getTree(contractAddress);
    return tree.contract.portalContract;
  }

  public async getFunctionAbi(contractAddress: AztecAddress, functionSelector: Buffer) {
    const tree = await this.getTree(contractAddress);
    return tree.getFunctionAbi(functionSelector);
  }

  public async getBytecode(contractAddress: AztecAddress, functionSelector: Buffer) {
    const tree = await this.getTree(contractAddress);
    return tree.getBytecode(functionSelector);
  }

  public async getContractMembershipWitness(contractAddress: AztecAddress) {
    const tree = await this.getTree(contractAddress);
    return tree.getContractMembershipWitness();
  }

  public async getFunctionMembershipWitness(contractAddress: AztecAddress, functionSelector: Buffer) {
    const tree = await this.getTree(contractAddress);
    return tree.getFunctionMembershipWitness(functionSelector);
  }

  public async getVkMembershipWitness(vk: VerificationKey) {
    // TODO
    return await Promise.resolve(MembershipWitness.random(VK_TREE_HEIGHT));
  }

  private async getTree(contractAddress: AztecAddress) {
    let tree = this.trees.find(t => t.contract.address.equals(contractAddress));
    if (!tree) {
      const contract = await this.db.getContract(contractAddress);
      if (!contract) {
        throw new Error(`Unknown contract: ${contractAddress}`);
      }

      const wasm = await CircuitsWasm.get();
      tree = new ContractTree(contract, this.node, wasm);
      this.trees.push(tree);
    }
    return tree;
  }
}
