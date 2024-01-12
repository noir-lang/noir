import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';

import { ContractData, ExtendedContractData } from './contract_data.js';

describe('ContractData', () => {
  const aztecAddress = AztecAddress.random();
  const portalAddress = EthAddress.random();

  it('serializes / deserializes correctly', () => {
    const extendedContractData = ExtendedContractData.random();
    const buf = extendedContractData.toBuffer();
    const serContractData = ExtendedContractData.fromBuffer(buf);
    expect(extendedContractData.contractData.contractAddress.equals(serContractData.contractData.contractAddress)).toBe(
      true,
    );
    expect(
      extendedContractData.contractData.portalContractAddress.equals(
        serContractData.contractData.portalContractAddress,
      ),
    ).toBe(true);
    expect(extendedContractData.bytecode?.equals(serContractData?.bytecode || Buffer.alloc(0))).toBe(true);
  });

  it('serializes / deserializes correctly without bytecode', () => {
    const contractData = new ContractData(aztecAddress, portalAddress);
    const buf = contractData.toBuffer();
    const serContractData = ContractData.fromBuffer(buf);
    expect(contractData.contractAddress.equals(serContractData.contractAddress)).toBe(true);
    expect(contractData.portalContractAddress.equals(serContractData.portalContractAddress)).toBe(true);
  });
});
