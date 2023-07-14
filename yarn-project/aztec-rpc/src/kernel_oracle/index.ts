import { AztecAddress, Fr, MembershipWitness, PRIVATE_DATA_TREE_HEIGHT } from '@aztec/circuits.js';
import { Tuple } from '@aztec/foundation/serialize';
import { AztecNode, MerkleTreeId } from '@aztec/types';

import { ContractDataOracle } from '../contract_data_oracle/index.js';
import { ProvingDataOracle } from './../kernel_prover/proving_data_oracle.js';

/**
 * A data oracle that provides information needed for simulating a transaction.
 */
export class KernelOracle implements ProvingDataOracle {
  constructor(private contractDataOracle: ContractDataOracle, private node: AztecNode) {}

  /**
   * Retrieves the contract membership witness for a given contract address.
   * A contract membership witness is a cryptographic proof that the contract exists in the Aztec network.
   * This function will search for an existing contract tree associated with the contract address and obtain its
   * membership witness. If no such contract tree exists, it will throw an error.
   *
   * @param contractAddress - The contract address.
   * @returns A promise that resolves to a MembershipWitness instance representing the contract membership witness.
   * @throws Error if the contract address is unknown or not found.
   */
  public async getContractMembershipWitness(contractAddress: AztecAddress) {
    return await this.contractDataOracle.getContractMembershipWitness(contractAddress);
  }

  /**
   * Retrieve the function membership witness for the given contract address and function selector.
   * The function membership witness represents a proof that the function belongs to the specified contract.
   * Throws an error if the contract address or function selector is unknown.
   *
   * @param contractAddress - The contract address.
   * @param functionSelector - The buffer containing the function selector.
   * @returns A promise that resolves with the MembershipWitness instance for the specified contract's function.
   */
  public async getFunctionMembershipWitness(contractAddress: AztecAddress, functionSelector: Buffer) {
    return await this.contractDataOracle.getFunctionMembershipWitness(contractAddress, functionSelector);
  }

  /**
   * Retrieve the membership witness corresponding to a verification key.
   * This function currently returns a random membership witness of the specified height,
   * which is a placeholder implementation until a concrete membership witness calculation
   * is implemented.
   *
   * @param vk - The VerificationKey for which the membership witness is needed.
   * @returns A Promise that resolves to the MembershipWitness instance.
   */
  public async getVkMembershipWitness() {
    return await this.contractDataOracle.getVkMembershipWitness();
  }

  /**
   * Get the note membership witness for a note in the private data tree at the given leaf index.
   *
   * @param leafIndex - The leaf index of the note in the private data tree.
   * @returns the MembershipWitness for the note.
   */
  async getNoteMembershipWitness(leafIndex: bigint): Promise<MembershipWitness<typeof PRIVATE_DATA_TREE_HEIGHT>> {
    const path = await this.node.getDataTreePath(leafIndex);
    return new MembershipWitness<typeof PRIVATE_DATA_TREE_HEIGHT>(
      path.pathSize,
      leafIndex,
      path.toFieldArray() as Tuple<Fr, typeof PRIVATE_DATA_TREE_HEIGHT>,
    );
  }

  /**
   * Get the root of the private data tree.
   *
   * @returns the root of the private data tree.
   */
  async getPrivateDataRoot(): Promise<Fr> {
    const roots = await this.node.getTreeRoots();
    return roots[MerkleTreeId.PRIVATE_DATA_TREE];
  }
}
