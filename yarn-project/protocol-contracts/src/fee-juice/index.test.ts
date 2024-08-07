import { computeContractAddressFromInstance, getContractClassFromArtifact } from '@aztec/circuits.js';

import { getCanonicalFeeJuice } from './index.js';

describe('FeeJuice', () => {
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalFeeJuice();
    expect(computeContractAddressFromInstance(contract.instance)).toEqual(contract.address);
    expect(getContractClassFromArtifact(contract.artifact).id).toEqual(contract.contractClass.id);
  });
});
