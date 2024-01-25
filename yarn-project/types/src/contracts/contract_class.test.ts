import { SerializableContractClass } from './contract_class.js';

describe('ContractClass', () => {
  it('can serialize and deserialize a contract class', () => {
    const contractClass = SerializableContractClass.random();
    expect(SerializableContractClass.fromBuffer(contractClass.toBuffer())).toEqual(contractClass);
  });
});
