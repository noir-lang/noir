import { computeContractAddressFromInstance, getContractClassFromArtifact } from '@aztec/circuits.js';

import { getCanonicalClassRegisterer, getCanonicalClassRegistererAddress } from './index.js';

describe('ClassRegisterer', () => {
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalClassRegisterer();
    expect(computeContractAddressFromInstance(contract.instance)).toEqual(contract.address);
    expect(getContractClassFromArtifact(contract.artifact).id).toEqual(contract.contractClass.id);
    expect(contract.address.toString()).toEqual(getCanonicalClassRegistererAddress().toString());
  });
});
