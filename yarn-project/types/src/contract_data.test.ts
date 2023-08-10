import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';

import { ContractData, ContractDataAndBytecode, EncodedContractFunction } from './contract_data.js';

describe('ContractData', () => {
  const aztecAddress = AztecAddress.random();
  const portalAddress = EthAddress.random();

  it('serializes / deserializes correctly', () => {
    const contractDataAndBytecode = new ContractDataAndBytecode(new ContractData(aztecAddress, portalAddress), [
      EncodedContractFunction.random(),
      EncodedContractFunction.random(),
    ]);
    const buf = contractDataAndBytecode.toBuffer();
    const serContractData = ContractDataAndBytecode.fromBuffer(buf);
    expect(
      contractDataAndBytecode.contractData.contractAddress.equals(serContractData.contractData.contractAddress),
    ).toBe(true);
    expect(
      contractDataAndBytecode.contractData.portalContractAddress.equals(
        serContractData.contractData.portalContractAddress,
      ),
    ).toBe(true);
    expect(contractDataAndBytecode.bytecode?.equals(serContractData?.bytecode || Buffer.alloc(0))).toBe(true);
  });

  it('serializes / deserializes correctly without bytecode', () => {
    const contractData = new ContractData(aztecAddress, portalAddress);
    const buf = contractData.toBuffer();
    const serContractData = ContractData.fromBuffer(buf);
    expect(contractData.contractAddress.equals(serContractData.contractAddress)).toBe(true);
    expect(contractData.portalContractAddress.equals(serContractData.portalContractAddress)).toBe(true);
  });
});
