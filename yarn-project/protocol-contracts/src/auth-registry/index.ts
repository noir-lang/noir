import { AztecAddress, CANONICAL_AUTH_REGISTRY_ADDRESS } from '@aztec/circuits.js';

import { type ProtocolContract, getCanonicalProtocolContract } from '../protocol_contract.js';
import { AuthRegistryArtifact } from './artifact.js';

/** Returns the canonical deployment of the auth registry. */
export function getCanonicalAuthRegistry(): ProtocolContract {
  const contract = getCanonicalProtocolContract(AuthRegistryArtifact, 1);

  if (!contract.address.equals(AuthRegistryAddress)) {
    throw new Error(
      `Incorrect address for auth registry (got ${contract.address.toString()} but expected ${AuthRegistryAddress.toString()}). Check CANONICAL_AUTH_REGISTRY_ADDRESS is set to the correct value in the constants files and run the protocol-contracts package tests.`,
    );
  }
  return contract;
}

export function getCanonicalAuthRegistryAddress(): AztecAddress {
  return getCanonicalAuthRegistry().address;
}

export const AuthRegistryAddress = AztecAddress.fromBigInt(CANONICAL_AUTH_REGISTRY_ADDRESS);
