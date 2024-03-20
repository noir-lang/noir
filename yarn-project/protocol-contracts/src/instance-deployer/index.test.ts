import { computeContractAddressFromInstance, getContractClassFromArtifact } from '@aztec/circuits.js';
import { setupCustomSnapshotSerializers } from '@aztec/foundation/testing';

import { InstanceDeployerAddress, getCanonicalInstanceDeployer } from './index.js';

describe('InstanceDeployer', () => {
  setupCustomSnapshotSerializers(expect);
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalInstanceDeployer();
    expect(computeContractAddressFromInstance(contract.instance)).toEqual(contract.address);
    expect(getContractClassFromArtifact(contract.artifact).id).toEqual(contract.contractClass.id);
    expect(contract.address.toString()).toEqual(InstanceDeployerAddress.toString());
  });
});
