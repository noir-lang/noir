import { NEW_CONTRACT_DATA_LENGTH } from '../../constants.gen.js';
import { makeNewContractData } from '../../tests/factories.js';
import { NewContractData } from './new_contract_data.js';

describe('NewContractData', () => {
  let data: NewContractData;

  beforeAll(() => {
    const randomInt = Math.floor(Math.random() * 1000);
    data = makeNewContractData(randomInt);
  });

  it('serializes to buffer and deserializes it back', () => {
    const buffer = data.toBuffer();
    const res = NewContractData.fromBuffer(buffer);
    expect(res).toEqual(data);
  });

  it('number of fields matches constant', () => {
    const fields = data.toFields();
    expect(fields.length).toBe(NEW_CONTRACT_DATA_LENGTH);
  });

  it('computes contract leaf', () => {
    const cd = makeNewContractData(12);
    const res = cd.computeLeaf();
    expect(res).toMatchSnapshot();
  });
});
