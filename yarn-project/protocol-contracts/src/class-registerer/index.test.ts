import omit from 'lodash.omit';

import { ClassRegistererAddress, getCanonicalClassRegisterer } from './index.js';

describe('ClassRegisterer', () => {
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalClassRegisterer();
    contract.contractClass.privateFunctions.sort((a, b) => a.selector.value - b.selector.value);
    contract.contractClass.publicFunctions.sort((a, b) => a.selector.value - b.selector.value);
    expect(omit(contract, 'artifact')).toMatchSnapshot();
    expect(contract.address.toString()).toEqual(ClassRegistererAddress.toString());
  });
});
