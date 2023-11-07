import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';
import { computeSecretMessageHash } from '@aztec/circuits.js/abis';
import { ContractArtifact, FunctionSelector, getFunctionDebugMetadata } from '@aztec/foundation/abi';
import { sha256ToField } from '@aztec/foundation/crypto';
import { L1Actor, L1ToL2Message, L2Actor } from '@aztec/types';

import { FunctionArtifactWithDebugMetadata } from '../index.js';

/**
 * Test utility function to craft an L1 to L2 message.
 * @param selector - The cross chain message selector.
 * @param contentPreimage - The args after the selector.
 * @param targetContract - The contract to consume the message.
 * @param secret - The secret to unlock the message.
 * @returns The L1 to L2 message.
 */
export const buildL1ToL2Message = (
  selector: string,
  contentPreimage: Fr[],
  targetContract: AztecAddress,
  secret: Fr,
) => {
  // Write the selector into a buffer.
  const selectorBuf = Buffer.from(selector, 'hex');

  const contentBuf = Buffer.concat([selectorBuf, ...contentPreimage.map(field => field.toBuffer())]);
  const content = sha256ToField(contentBuf);

  const secretHash = computeSecretMessageHash(secret);

  // Eventually the kernel will need to prove the kernel portal pair exists within the contract tree,
  // EthAddress.random() will need to be replaced when this happens
  return new L1ToL2Message(
    new L1Actor(EthAddress.random(), 1),
    new L2Actor(targetContract, 1),
    content,
    secretHash,
    0,
    0,
  );
};

export const getFunctionArtifact = (
  artifact: ContractArtifact,
  functionName: string,
): FunctionArtifactWithDebugMetadata => {
  const functionArtifact = artifact.functions.find(f => f.name === functionName);
  if (!functionArtifact) {
    throw new Error(`Unknown function ${functionName}`);
  }

  const debug = getFunctionDebugMetadata(artifact, functionName);
  return { ...functionArtifact, debug };
};

export const getFunctionArtifactWithSelector = (
  artifact: ContractArtifact,
  functionSelector: FunctionSelector,
): FunctionArtifactWithDebugMetadata => {
  const functionArtifact = artifact.functions.find(f =>
    functionSelector.equals(FunctionSelector.fromNameAndParameters(f.name, f.parameters)),
  );
  if (!functionArtifact) {
    throw new Error(`Unknown function ${functionSelector}`);
  }

  const debug = getFunctionDebugMetadata(artifact, functionArtifact.name);
  return { ...functionArtifact, debug };
};
