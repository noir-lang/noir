import { makeContractStorageRead } from '../tests/factories.js';
import { ContractStorageRead } from './contract_storage_read.js';

describe('ContractStorageRead', () => {
  it('serializes to buffer and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makeContractStorageRead(randomInt);
    const buffer = expected.toBuffer();
    const res = ContractStorageRead.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('serializes to field array and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makeContractStorageRead(randomInt);

    const fieldArray = expected.toFields();
    const res = ContractStorageRead.fromFields(fieldArray);
    expect(res).toEqual(expected);
  });
});
