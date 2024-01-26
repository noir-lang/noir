import { AztecNode, KeyStore, MerkleTreeId } from '@aztec/circuit-types';
import {
  AztecAddress,
  Fr,
  FunctionSelector,
  MembershipWitness,
  NOTE_HASH_TREE_HEIGHT,
  Point,
} from '@aztec/circuits.js';
import { Tuple } from '@aztec/foundation/serialize';

import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { ProvingDataOracle } from './../kernel_prover/proving_data_oracle.js';

/**
 * A data oracle that provides information needed for simulating a transaction.
 */
export class KernelOracle implements ProvingDataOracle {
  constructor(private contractDataOracle: ContractDataOracle, private keyStore: KeyStore, private node: AztecNode) {}

  public async getContractMembershipWitness(contractAddress: AztecAddress) {
    return await this.contractDataOracle.getContractMembershipWitness(contractAddress);
  }

  public async getFunctionMembershipWitness(contractAddress: AztecAddress, selector: FunctionSelector) {
    return await this.contractDataOracle.getFunctionMembershipWitness(contractAddress, selector);
  }

  public async getVkMembershipWitness() {
    return await this.contractDataOracle.getVkMembershipWitness();
  }

  async getNoteMembershipWitness(leafIndex: bigint): Promise<MembershipWitness<typeof NOTE_HASH_TREE_HEIGHT>> {
    const path = await this.node.getNoteHashSiblingPath('latest', leafIndex);
    return new MembershipWitness<typeof NOTE_HASH_TREE_HEIGHT>(
      path.pathSize,
      leafIndex,
      path.toFieldArray() as Tuple<Fr, typeof NOTE_HASH_TREE_HEIGHT>,
    );
  }

  async getNoteHashTreeRoot(): Promise<Fr> {
    const roots = await this.node.getTreeRoots();
    return roots[MerkleTreeId.NOTE_HASH_TREE];
  }

  public getMasterNullifierSecretKey(nullifierPublicKey: Point) {
    return this.keyStore.getNullifierSecretKeyFromPublicKey(nullifierPublicKey);
  }
}
