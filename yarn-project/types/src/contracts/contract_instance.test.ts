import { SerializableContractInstance } from './contract_instance.js';

describe('ContractInstance', () => {
  it('can serialize and deserialize an instance', () => {
    const instance = SerializableContractInstance.random();
    expect(SerializableContractInstance.fromBuffer(instance.toBuffer())).toEqual(instance);
  });
});
