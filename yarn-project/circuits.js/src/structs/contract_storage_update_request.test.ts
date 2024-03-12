import { randomInt } from '@aztec/foundation/crypto';

import { CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH } from '../constants.gen.js';
import { makeContractStorageUpdateRequest } from '../tests/factories.js';
import { ContractStorageUpdateRequest } from './contract_storage_update_request.js';

describe('ContractStorageUpdateRequest', () => {
  let request: ContractStorageUpdateRequest;

  beforeAll(() => {
    request = makeContractStorageUpdateRequest(randomInt(1000));
  });

  it('serializes to buffer and deserializes it back', () => {
    const buffer = request.toBuffer();
    const res = ContractStorageUpdateRequest.fromBuffer(buffer);
    expect(res).toEqual(request);
  });

  it('serializes to field array and deserializes it back', () => {
    const fieldArray = request.toFields();
    const res = ContractStorageUpdateRequest.fromFields(fieldArray);
    expect(res).toEqual(request);
  });

  it('number of fields matches constant', () => {
    const fields = request.toFields();
    expect(fields.length).toBe(CONTRACT_STORAGE_UPDATE_REQUEST_LENGTH);
  });
});
