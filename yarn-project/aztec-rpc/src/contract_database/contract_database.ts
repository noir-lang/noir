import { AztecAddress } from '@aztec/circuits.js';
import { ContractDao } from './contract_dao.js';

export interface ContractDatabase {
  addContract(contract: ContractDao): Promise<void>;
  getContract(address: AztecAddress): Promise<ContractDao | undefined>;
}
