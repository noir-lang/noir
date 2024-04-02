import { MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS, getContractClassFromArtifact } from '@aztec/circuits.js';
import { type ContractArtifact, bufferAsFields } from '@aztec/foundation/abi';

import { type ContractFunctionInteraction } from '../contract/contract_function_interaction.js';
import { type Wallet } from '../wallet/index.js';
import { getRegistererContract } from './protocol_contracts.js';

/** Sets up a call to register a contract class given its artifact. */
export async function registerContractClass(
  wallet: Wallet,
  artifact: ContractArtifact,
): Promise<ContractFunctionInteraction> {
  const { artifactHash, privateFunctionsRoot, publicBytecodeCommitment, packedBytecode } =
    getContractClassFromArtifact(artifact);
  const encodedBytecode = bufferAsFields(packedBytecode, MAX_PACKED_PUBLIC_BYTECODE_SIZE_IN_FIELDS);
  const registerer = getRegistererContract(wallet);
  await wallet.addCapsule(encodedBytecode);
  return registerer.methods.register(artifactHash, privateFunctionsRoot, publicBytecodeCommitment);
}
