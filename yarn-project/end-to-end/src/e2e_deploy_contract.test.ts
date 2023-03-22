import { Aztec } from '@aztec/aztec.js';

describe('e2e_deploy_contract', () => {
  it('should deploy a contract', () => {
    expect(() => new Aztec()).not.toThrow();
  });
});
