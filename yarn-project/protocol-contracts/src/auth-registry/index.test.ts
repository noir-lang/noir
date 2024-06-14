import {
  AztecAddress,
  CANONICAL_AUTH_REGISTRY_ADDRESS,
  computeContractAddressFromInstance,
  getContractClassFromArtifact,
} from '@aztec/circuits.js';

import { getCanonicalAuthRegistry } from './index.js';

describe('AuthRegistry', () => {
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalAuthRegistry();
    expect(computeContractAddressFromInstance(contract.instance)).toEqual(contract.address);
    expect(getContractClassFromArtifact(contract.artifact).id).toEqual(contract.contractClass.id);
    expect(contract.address).toEqual(AztecAddress.fromBigInt(CANONICAL_AUTH_REGISTRY_ADDRESS));
  });
});
