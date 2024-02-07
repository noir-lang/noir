import omit from 'lodash.omit';

import { InstanceDeployerAddress, getCanonicalInstanceDeployer } from './index.js';

describe('InstanceDeployer', () => {
  it('returns canonical protocol contract', () => {
    // TODO(@spalladino): Consider sorting functions by selector when constructing the contract
    // class, or even better, when calling loadContractArtifact from the Noir output.
    const contract = getCanonicalInstanceDeployer();
    contract.contractClass.privateFunctions.sort((a, b) => a.selector.value - b.selector.value);
    contract.contractClass.publicFunctions.sort((a, b) => a.selector.value - b.selector.value);
    expect(omit(contract, 'artifact')).toMatchSnapshot();
    expect(contract.address.toString()).toEqual(InstanceDeployerAddress.toString());
  });
});
