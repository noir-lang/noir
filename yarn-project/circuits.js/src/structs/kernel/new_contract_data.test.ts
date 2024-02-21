import { Fr } from '@aztec/foundation/fields';

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
    const hash = cd.hash();
    expect(hash).toMatchSnapshot();
  });

  it('empty "hash" is zero', () => {
    const cd = NewContractData.empty();
    expect(cd.isEmpty()).toBe(true);

    const hash = cd.hash();
    expect(hash).toEqual(Fr.ZERO);
  });

  it('hash matches', () => {
    const cd = makeNewContractData(5);
    const hash = cd.hash();
    expect(hash).toMatchSnapshot();

    // Value used in hash_matches test in new_contract_data.nr
    // console.log("hash", hash.toString());
  });
});
