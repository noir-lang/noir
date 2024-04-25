import {
  AztecAddress,
  CANONICAL_KEY_REGISTRY_ADDRESS,
  computeContractAddressFromInstance,
  getContractClassFromArtifact,
} from '@aztec/circuits.js';

import { getCanonicalKeyRegistry } from './index.js';

describe('KeyRegistry', () => {
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalKeyRegistry();
    expect(computeContractAddressFromInstance(contract.instance)).toEqual(contract.address);
    expect(getContractClassFromArtifact(contract.artifact).id).toEqual(contract.contractClass.id);
    expect(contract.address).toEqual(AztecAddress.fromBigInt(CANONICAL_KEY_REGISTRY_ADDRESS));
  });
});
