import omit from 'lodash.omit';

import { GasTokenAddress, getCanonicalGasToken } from './index.js';

describe.skip('GasToken', () => {
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalGasToken();
    contract.contractClass.privateFunctions.sort((a, b) => a.selector.value - b.selector.value);
    contract.contractClass.publicFunctions.sort((a, b) => a.selector.value - b.selector.value);
    expect(omit(contract, 'artifact')).toMatchSnapshot();
    expect(contract.address.toString()).toEqual(GasTokenAddress.toString());
  });
});
