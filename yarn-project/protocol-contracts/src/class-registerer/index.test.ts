import omit from 'lodash.omit';

import { ClassRegistererAddress, getCanonicalClassRegisterer } from './index.js';

describe('ClassRegisterer', () => {
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalClassRegisterer();
    expect(omit(contract, 'artifact')).toMatchSnapshot();
    expect(contract.address.toString()).toEqual(ClassRegistererAddress.toString());
  });
});
