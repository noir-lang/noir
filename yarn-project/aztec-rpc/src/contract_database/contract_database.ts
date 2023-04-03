import { AztecAddress } from '@aztec/foundation';
import { ContractDao } from './contract_dao.js';

export interface ContractDatabase {
  addContract(contract: ContractDao): Promise<void>;
  getContract(address: AztecAddress): Promise<ContractDao | undefined>;
}
