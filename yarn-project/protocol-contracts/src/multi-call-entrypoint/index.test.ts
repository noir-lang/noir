import { computeContractAddressFromInstance, getContractClassFromArtifact } from '@aztec/circuits.js';

import { getCanonicalMultiCallEntrypointContract } from './index.js';

describe('MultiCallEntrypoint', () => {
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalMultiCallEntrypointContract();
    expect(computeContractAddressFromInstance(contract.instance)).toEqual(contract.address);
    expect(getContractClassFromArtifact(contract.artifact).id).toEqual(contract.contractClass.id);
  });
});
