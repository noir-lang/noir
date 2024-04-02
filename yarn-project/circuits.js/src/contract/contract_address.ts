import { type FunctionAbi, FunctionSelector, encodeArguments } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { type ContractInstance } from '@aztec/types/contracts';

import { GeneratorIndex } from '../constants.gen.js';
import { computeVarArgsHash } from '../hash/hash.js';
import { type PublicKey } from '../types/public_key.js';

// TODO(@spalladino): Review all generator indices in this file

/**
 * Returns the deployment address for a given contract instance as defined on the [Yellow Paper](../../../../yellow-paper/docs/addresses-and-keys/specification.md).
 * ```
 * salted_initialization_hash = pedersen([salt, initialization_hash, deployer, portal_contract_address as Field], GENERATOR__SALTED_INITIALIZATION_HASH)
 * partial_address = pedersen([contract_class_id, salted_initialization_hash], GENERATOR__CONTRACT_PARTIAL_ADDRESS_V1)
 * address = pedersen([public_keys_hash, partial_address], GENERATOR__CONTRACT_ADDRESS_V1)
 * ```
 * @param instance - A contract instance for which to calculate the deployment address.
 */
export function computeContractAddressFromInstance(
  instance:
    | ContractInstance
    | ({ contractClassId: Fr; saltedInitializationHash: Fr } & Pick<ContractInstance, 'publicKeysHash'>),
): AztecAddress {
  const partialAddress = computePartialAddress(instance);
  const publicKeyHash = instance.publicKeysHash;
  return computeContractAddressFromPartial({ partialAddress, publicKeyHash });
}

/**
 * Computes the partial address defined as the hash of the contract class id and salted initialization hash.
 * @param instance - Contract instance for which to calculate the partial address.
 */
export function computePartialAddress(
  instance:
    | Pick<ContractInstance, 'contractClassId' | 'initializationHash' | 'salt' | 'portalContractAddress' | 'deployer'>
    | { contractClassId: Fr; saltedInitializationHash: Fr },
): Fr {
  const saltedInitializationHash =
    'saltedInitializationHash' in instance
      ? instance.saltedInitializationHash
      : computeSaltedInitializationHash(instance);

  return pedersenHash([instance.contractClassId, saltedInitializationHash], GeneratorIndex.PARTIAL_ADDRESS);
}

/**
 * Computes the salted initialization hash for an address, defined as the hash of the salt, initialization hash, and portal address.
 * @param instance - Contract instance for which to compute the salted initialization hash.
 */
export function computeSaltedInitializationHash(
  instance: Pick<ContractInstance, 'initializationHash' | 'salt' | 'portalContractAddress' | 'deployer'>,
): Fr {
  return pedersenHash(
    [instance.salt, instance.initializationHash, instance.deployer, instance.portalContractAddress],
    GeneratorIndex.PARTIAL_ADDRESS,
  );
}

/**
 * Computes a contract address from its partial address and the pubkeys hash.
 * @param args - The hash of the public keys or the plain public key to be hashed, along with the partial address.
 * @returns The partially constructed contract address.
 */
export function computeContractAddressFromPartial(
  args: ({ publicKeyHash: Fr } | { publicKey: PublicKey }) & { partialAddress: Fr },
): AztecAddress {
  const publicKeyHash = 'publicKey' in args ? computePublicKeysHash(args.publicKey) : args.publicKeyHash;
  const result = pedersenHash([publicKeyHash, args.partialAddress], GeneratorIndex.CONTRACT_ADDRESS);
  return AztecAddress.fromField(result);
}

/**
 * Computes the hash of a set of public keys to be used for computing the deployment address of a contract.
 * @param publicKey - Single public key (for now!).
 * @returns The hash of the public keys.
 */
export function computePublicKeysHash(publicKey: PublicKey | undefined): Fr {
  if (!publicKey) {
    return Fr.ZERO;
  }
  return pedersenHash([publicKey.x, publicKey.y], GeneratorIndex.PARTIAL_ADDRESS);
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
