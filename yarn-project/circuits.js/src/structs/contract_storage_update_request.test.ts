import { makeContractStorageUpdateRequest } from '../tests/factories.js';
import { ContractStorageUpdateRequest } from './contract_storage_update_request.js';

describe('ContractStorageUpdateRequest', () => {
  it('serializes to buffer and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makeContractStorageUpdateRequest(randomInt);
    const buffer = expected.toBuffer();
    const res = ContractStorageUpdateRequest.fromBuffer(buffer);
    expect(res).toEqual(expected);
  });

  it('serializes to field array and deserializes it back', () => {
    const randomInt = Math.floor(Math.random() * 1000);
    const expected = makeContractStorageUpdateRequest(randomInt);

    const fieldArray = expected.toFields();
    const res = ContractStorageUpdateRequest.fromFields(fieldArray);
    expect(res).toEqual(expected);
  });
});
