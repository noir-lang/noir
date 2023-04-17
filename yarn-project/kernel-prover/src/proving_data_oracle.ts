import {
  CONTRACT_TREE_HEIGHT,
  FUNCTION_TREE_HEIGHT,
  MembershipWitness,
  VK_TREE_HEIGHT,
  VerificationKey,
} from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation';

export interface ProvingDataOracle {
  getVkMembershipWitness(vk: VerificationKey): Promise<MembershipWitness<typeof VK_TREE_HEIGHT>>;
  getContractMembershipWitness(contractAddress: AztecAddress): Promise<MembershipWitness<typeof CONTRACT_TREE_HEIGHT>>;
  getFunctionMembershipWitness(
    contractAddress: AztecAddress,
    functionSelector: Buffer,
  ): Promise<MembershipWitness<typeof FUNCTION_TREE_HEIGHT>>;
}
