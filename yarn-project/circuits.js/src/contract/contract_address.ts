import { type FunctionAbi, FunctionSelector, encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { pedersenHash, poseidon2Hash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { type ContractInstance } from '@aztec/types/contracts';

import { GeneratorIndex } from '../constants.gen.js';
import { computeVarArgsHash } from '../hash/hash.js';
import { deriveKeys } from '../keys/index.js';

// TODO(@spalladino): Review all generator indices in this file

/**
 * Returns the deployment address for a given contract instance as defined on the [Protocol Specs](../../../../docs/docs/protocol-specs/addresses-and-keys/specification.md).
 * ```
 * salted_initialization_hash = pedersen([salt, initialization_hash, deployer], GENERATOR__SALTED_INITIALIZATION_HASH)
 * partial_address = pedersen([contract_class_id, salted_initialization_hash], GENERATOR__CONTRACT_PARTIAL_ADDRESS_V1)
 * address = poseidon2Hash([public_keys_hash, partial_address, GENERATOR__CONTRACT_ADDRESS_V1])
 * ```
 * @param instance - A contract instance for which to calculate the deployment address.
 */
export function computeContractAddressFromInstance(
  instance:
    | ContractInstance
    | ({ contractClassId: Fr; saltedInitializationHash: Fr } & Pick<ContractInstance, 'publicKeysHash'>),
): AztecAddress {
  const partialAddress = computePartialAddress(instance);
  const publicKeysHash = instance.publicKeysHash;
  return computeContractAddressFromPartial({ partialAddress, publicKeysHash });
}

/**
 * Computes the partial address defined as the hash of the contract class id and salted initialization hash.
 * @param instance - Contract instance for which to calculate the partial address.
 */
export function computePartialAddress(
  instance:
    | Pick<ContractInstance, 'contractClassId' | 'initializationHash' | 'salt' | 'deployer'>
    | { contractClassId: Fr; saltedInitializationHash: Fr },
): Fr {
  const saltedInitializationHash =
    'saltedInitializationHash' in instance
      ? instance.saltedInitializationHash
      : computeSaltedInitializationHash(instance);

  return pedersenHash([instance.contractClassId, saltedInitializationHash], GeneratorIndex.PARTIAL_ADDRESS);
}

/**
 * Computes the salted initialization hash for an address, defined as the hash of the salt and initialization hash.
 * @param instance - Contract instance for which to compute the salted initialization hash.
 */
export function computeSaltedInitializationHash(
  instance: Pick<ContractInstance, 'initializationHash' | 'salt' | 'deployer'>,
): Fr {
  return pedersenHash([instance.salt, instance.initializationHash, instance.deployer], GeneratorIndex.PARTIAL_ADDRESS);
}

/**
 * Computes a contract address from its partial address and public keys hash.
 * @param args - The hash of the public keys or the plain public key to be hashed, along with the partial address.
 * @returns The contract address.
 */
export function computeContractAddressFromPartial(
  args: ({ publicKeysHash: Fr } | { secretKey: Fr }) & { partialAddress: Fr },
): AztecAddress {
  const publicKeysHash = 'secretKey' in args ? deriveKeys(args.secretKey).publicKeysHash : args.publicKeysHash;
  const result = poseidon2Hash([publicKeysHash, args.partialAddress, GeneratorIndex.CONTRACT_ADDRESS_V1]);
  return AztecAddress.fromField(result);
}

/**
 * Computes the initialization hash for an instance given its constructor function and arguments.
 * @param initFn - Constructor function or empty if no initialization is expected.
 * @param args - Unencoded arguments, will be encoded as fields according to the constructor function abi.
 * @returns The hash, or zero if no initialization function is provided.
 */
export function computeInitializationHash(initFn: FunctionAbi | undefined, args: any[]): Fr {
  if (!initFn) {
    return Fr.ZERO;
  }
  const selector = FunctionSelector.fromNameAndParameters(initFn.name, initFn.parameters);
  const flatArgs = encodeArguments(initFn, args);
  return computeInitializationHashFromEncodedArgs(selector, flatArgs);
}

/**
 * Computes the initialization hash for an instance given its constructor function selector and encoded arguments.
 * @param initFn - Constructor function selector.
 * @param args - Encoded arguments.
 * @returns The hash.
 */
export function computeInitializationHashFromEncodedArgs(initFn: FunctionSelector, encodedArgs: Fr[]): Fr {
  const argsHash = computeVarArgsHash(encodedArgs);
  return pedersenHash([initFn, argsHash], GeneratorIndex.CONSTRUCTOR);
}
