import { ContractAbi } from '@aztec/foundation/abi';

/**
 * Checks if the given input looks like a valid ContractAbi. The check is not exhaustive,
 * and it's just meant to differentiate between nargo raw build artifacts and the ones
 * produced by this compiler.
 * @param input - Input object.
 * @returns True if it looks like a ContractAbi.
 */
export function isContractAbi(input: any): input is ContractAbi {
  if (typeof input !== 'object') return false;
  const maybeContractAbi = input as ContractAbi;
  if (typeof maybeContractAbi.name !== 'string') return false;
  if (!Array.isArray(maybeContractAbi.functions)) return false;
  for (const fn of maybeContractAbi.functions) {
    if (typeof fn.name !== 'string') return false;
    if (typeof fn.functionType !== 'string') return false;
    if (typeof fn.isInternal !== 'boolean') return false;
  }
  return true;
}
