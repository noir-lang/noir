import { randomInt } from '@aztec/foundation/crypto';

import { CONTRACT_STORAGE_READ_LENGTH } from '../constants.gen.js';
import { makeContractStorageRead } from '../tests/factories.js';
import { ContractStorageRead } from './contract_storage_read.js';

describe('ContractStorageRead', () => {
  let read: ContractStorageRead;

  beforeAll(() => {
    read = makeContractStorageRead(randomInt(1000));
  });

  it('serializes to buffer and deserializes it back', () => {
    const buffer = read.toBuffer();
    const res = ContractStorageRead.fromBuffer(buffer);
    expect(res).toEqual(read);
  });

  it('serializes to field array and deserializes it back', () => {
    const fieldArray = read.toFields();
    const res = ContractStorageRead.fromFields(fieldArray);
    expect(res).toEqual(read);
  });

  it('number of fields matches constant', () => {
    const fields = read.toFields();
    expect(fields.length).toBe(CONTRACT_STORAGE_READ_LENGTH);
  });
});
