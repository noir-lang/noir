import { EthAddress, computeContractAddressFromInstance, getContractClassFromArtifact } from '@aztec/circuits.js';

import { getCanonicalGasToken } from './index.js';

describe('GasToken', () => {
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalGasToken(EthAddress.ZERO);
    expect(computeContractAddressFromInstance(contract.instance)).toEqual(contract.address);
    expect(getContractClassFromArtifact(contract.artifact).id).toEqual(contract.contractClass.id);
  });
});
