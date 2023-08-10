import {
  AztecAddress,
  CircuitsWasm,
  ConstantHistoricBlockData,
  Fr,
  GlobalVariables,
  MembershipWitness,
  PRIVATE_DATA_TREE_HEIGHT,
} from '@aztec/circuits.js';
import { computeGlobalsHash } from '@aztec/circuits.js/abis';
import { Tuple } from '@aztec/foundation/serialize';
import { AztecNode, MerkleTreeId } from '@aztec/types';

import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { ProvingDataOracle } from './../kernel_prover/proving_data_oracle.js';

/**
 * A data oracle that provides information needed for simulating a transaction.
 */
export class KernelOracle implements ProvingDataOracle {
  constructor(private contractDataOracle: ContractDataOracle, private node: AztecNode) {}

  public async getContractMembershipWitness(contractAddress: AztecAddress) {
    return await this.contractDataOracle.getContractMembershipWitness(contractAddress);
  }

  public async getFunctionMembershipWitness(contractAddress: AztecAddress, functionSelector: Buffer) {
    return await this.contractDataOracle.getFunctionMembershipWitness(contractAddress, functionSelector);
  }

  public async getVkMembershipWitness() {
    return await this.contractDataOracle.getVkMembershipWitness();
  }

  async getNoteMembershipWitness(leafIndex: bigint): Promise<MembershipWitness<typeof PRIVATE_DATA_TREE_HEIGHT>> {
    const path = await this.node.getDataTreePath(leafIndex);
    return new MembershipWitness<typeof PRIVATE_DATA_TREE_HEIGHT>(
      path.pathSize,
      leafIndex,
      path.toFieldArray() as Tuple<Fr, typeof PRIVATE_DATA_TREE_HEIGHT>,
    );
  }

  async getPrivateDataRoot(): Promise<Fr> {
    const roots = await this.node.getTreeRoots();
    return roots[MerkleTreeId.PRIVATE_DATA_TREE];
  }

  async getConstantHistoricBlockData(): Promise<ConstantHistoricBlockData> {
    const wasm = await CircuitsWasm.get();
    const latestBlock = await this.node.getBlock(-1);
    const latestGlobals = latestBlock?.globalVariables ?? GlobalVariables.empty();
    const prevBlockGlobalVariablesHash = computeGlobalsHash(wasm, latestGlobals);
    const treeRoots = await this.node.getTreeRoots();

    return new ConstantHistoricBlockData(
      treeRoots[MerkleTreeId.PRIVATE_DATA_TREE],
      treeRoots[MerkleTreeId.NULLIFIER_TREE],
      treeRoots[MerkleTreeId.CONTRACT_TREE],
      treeRoots[MerkleTreeId.L1_TO_L2_MESSAGES_TREE],
      treeRoots[MerkleTreeId.BLOCKS_TREE],
      Fr.ZERO,
      treeRoots[MerkleTreeId.PUBLIC_DATA_TREE],
      prevBlockGlobalVariablesHash,
    );
  }
}
