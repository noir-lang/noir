import { type ContractArtifact } from '@aztec/foundation/abi';

/**
 * Checks if the given input looks like a valid ContractArtifact. The check is not exhaustive,
 * and it's just meant to differentiate between nargo raw build artifacts and the ones
 * produced by this compiler.
 * @param input - Input object.
 * @returns True if it looks like a ContractArtifact.
 */
export function isContractArtifact(input: any): input is ContractArtifact {
  if (typeof input !== 'object') {
    return false;
  }
  const maybeContractArtifact = input as ContractArtifact;
  if (typeof maybeContractArtifact.name !== 'string') {
    return false;
  }
  if (!Array.isArray(maybeContractArtifact.functions)) {
    return false;
  }
  for (const fn of maybeContractArtifact.functions) {
    if (typeof fn.name !== 'string') {
      return false;
    }
    if (typeof fn.functionType !== 'string') {
      return false;
    }
    if (typeof fn.isInternal !== 'boolean') {
      return false;
    }
  }
  return true;
}
