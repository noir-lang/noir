import { GasTokenAddress, getCanonicalGasToken } from './index.js';

describe('GasToken', () => {
  it('returns canonical protocol contract', () => {
    const contract = getCanonicalGasToken();
    expect(contract.address.toString()).toEqual(GasTokenAddress.toString());
  });
});
