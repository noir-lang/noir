import { AztecAddress, EthAddress } from '@aztec/foundation';
import { ContractData, EncodedContractFunction } from './contract_data.js';

describe('ContractData', () => {
  const aztecAddress = AztecAddress.random();
  const portalAddress = EthAddress.random();

  it('serializes / deserializes correctly', () => {
    const contractData = new ContractData(aztecAddress, portalAddress, [
      EncodedContractFunction.random(),
      EncodedContractFunction.random(),
    ]);
    const buf = contractData.toBuffer();
    const serContractData = ContractData.fromBuffer(buf);
    expect(contractData.contractAddress.equals(serContractData.contractAddress)).toBe(true);
    expect(contractData.portalContractAddress.equals(serContractData.portalContractAddress)).toBe(true);
    expect(contractData.bytecode?.equals(serContractData?.bytecode || Buffer.alloc(0))).toBe(true);
  });

  it('serializes / deserializes correctly without bytecode', () => {
    const contractData = new ContractData(aztecAddress, portalAddress);
    const buf = contractData.toBuffer();
    const serContractData = ContractData.fromBuffer(buf);
    expect(contractData.contractAddress.equals(serContractData.contractAddress)).toBe(true);
    expect(contractData.portalContractAddress.equals(serContractData.portalContractAddress)).toBe(true);
  });
});
