import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';

import { ContractData } from './contract_data.js';

describe('ContractData', () => {
  const aztecAddress = AztecAddress.random();
  const portalAddress = EthAddress.random();

  it('serializes / deserializes correctly without bytecode', () => {
    const contractData = new ContractData(aztecAddress, portalAddress);
    const buf = contractData.toBuffer();
    const serContractData = ContractData.fromBuffer(buf);
    expect(contractData.contractAddress.equals(serContractData.contractAddress)).toBe(true);
    expect(contractData.portalContractAddress.equals(serContractData.portalContractAddress)).toBe(true);
  });
});
