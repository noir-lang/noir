import { AztecAddress } from '@aztec/foundation/aztec-address';
import { ContractDao } from './contract_dao.js';
import { ContractDatabase } from './contract_database.js';

export class MemoryContractDatabase implements ContractDatabase {
  private contracts: ContractDao[] = [];

  public addContract(contract: ContractDao) {
    this.contracts.push(contract);
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
