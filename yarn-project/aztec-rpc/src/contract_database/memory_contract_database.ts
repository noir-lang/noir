import { AztecAddress, EthAddress } from '@aztec/circuits.js';
import { ContractAbi } from '../noir.js';
import { contractAbiToContractDao, ContractDao } from './contract_dao.js';
import { ContractDataSource } from './contract_database.js';

export class MemoryContractDatabase implements ContractDataSource {
  private contracts: ContractDao[] = [];

  public addContract(address: AztecAddress, portalAddress: EthAddress, abi: ContractAbi) {
    this.contracts.push(contractAbiToContractDao(address, portalAddress, abi));
    return Promise.resolve();
  }

  public getContract(address: AztecAddress) {
    return Promise.resolve(this.contracts.find(c => c.address.equals(address)));
  }

  public async getCode(contractAddress: AztecAddress, functionSelector: Buffer) {
    const contract = await this.getContract(contractAddress);
    return contract?.functions.find(f => f.selector.equals(functionSelector))?.bytecode;
  }
}
