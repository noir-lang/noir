import {
  CONTRACT_TREE_HEIGHT,
  FUNCTION_TREE_HEIGHT,
  Fr,
  MembershipWitness,
  PRIVATE_DATA_TREE_HEIGHT,
  VK_TREE_HEIGHT,
  VerificationKey,
} from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';

/**
 * Provides functionality to fetch membership witnesses for verification keys,
 * contract addresses, and function selectors in their respective merkle trees.
 */
export interface ProvingDataOracle {
  getContractMembershipWitness(contractAddress: AztecAddress): Promise<MembershipWitness<typeof CONTRACT_TREE_HEIGHT>>;
  getFunctionMembershipWitness(
    contractAddress: AztecAddress,
    functionSelector: Buffer,
  ): Promise<MembershipWitness<typeof FUNCTION_TREE_HEIGHT>>;
  getVkMembershipWitness(vk: VerificationKey): Promise<MembershipWitness<typeof VK_TREE_HEIGHT>>;
  getNoteMembershipWitness(leafIndex: bigint): Promise<MembershipWitness<typeof PRIVATE_DATA_TREE_HEIGHT>>;
  getPrivateDataRoot(): Promise<Fr>;
}
