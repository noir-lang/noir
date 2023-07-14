import { AztecAddress } from '@aztec/foundation/aztec-address';

import { ContractDao } from './contract_dao.js';

/**
 * Represents a ContractDatabase interface for managing noir contracts.
 * Provides methods for adding and retrieving ContractDao objects by their associated addresses.
 */
export interface ContractDatabase {
  addContract(contract: ContractDao): Promise<void>;
  getContract(address: AztecAddress): Promise<ContractDao | undefined>;
}
