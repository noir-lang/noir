import { ContractArtifact, FunctionSelector, FunctionType } from '@aztec/foundation/abi';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { ContractClass } from '@aztec/types/contracts';

import chunk from 'lodash.chunk';

import { GeneratorIndex } from '../constants.gen.js';

/** Contract artifact including its artifact hash */
type ContractArtifactWithHash = ContractArtifact & { artifactHash: Fr };

/**
 * Creates a ContractClass from a contract compilation artifact with its artifact hash.
 */
export function createContractClassFromArtifact(artifact: ContractArtifactWithHash): ContractClass {
  return {
    version: 1,
    artifactHash: artifact.artifactHash,
    publicFunctions: artifact.functions
      .filter(f => f.functionType === FunctionType.OPEN)
      .map(f => ({
        selector: FunctionSelector.fromNameAndParameters(f.name, f.parameters),
        bytecode: Buffer.from(f.bytecode, 'base64'),
        isInternal: f.isInternal,
      })),
    privateFunctions: artifact.functions
      .filter(f => f.functionType === FunctionType.SECRET)
      .map(f => ({
        selector: FunctionSelector.fromNameAndParameters(f.name, f.parameters),
        vkHash: getVerificationKeyHash(Buffer.from(f.verificationKey!, 'base64')),
        isInternal: f.isInternal,
      })),
    packedBytecode: Buffer.alloc(0),
  };
}

/**
 * Calculates the hash of a verification key.
 * TODO(@spalladino) Check this is the correct calculation of vkhash
 * */
function getVerificationKeyHash(vk: Buffer) {
  const chunks = chunk(vk, 32).map(nums => Buffer.from(nums));
  return Fr.fromBuffer(pedersenHash(chunks, GeneratorIndex.VK));
}
