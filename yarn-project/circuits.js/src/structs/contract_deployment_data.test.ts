import { makeContractDeploymentData } from '../tests/factories.js';
import { ContractDeploymentData } from './contract_deployment_data.js';

describe('ContractDeploymentData', () => {
  it(`serializes to buffer and deserializes it back`, () => {
    const expected = makeContractDeploymentData(1);
    const buffer = expected.toBuffer();
    const res = ContractDeploymentData.fromBuffer(buffer);
    expect(res).toEqual(expected);
    expect(res.isEmpty()).toBe(false);
  });

  it(`initializes an empty ContractDeploymentData`, () => {
    const target = ContractDeploymentData.empty();
    expect(target.isEmpty()).toBe(true);
  });
});
