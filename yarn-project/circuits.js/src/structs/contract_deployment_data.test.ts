import { CONTRACT_DEPLOYMENT_DATA_LENGTH } from '../constants.gen.js';
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

  it('number of fields matches constant', () => {
    const target = makeContractDeploymentData(327);
    const fields = target.toFields();
    expect(fields.length).toBe(CONTRACT_DEPLOYMENT_DATA_LENGTH);
  });
});
