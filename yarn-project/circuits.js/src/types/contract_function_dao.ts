import { FunctionAbi } from '@aztec/foundation/abi';

import { FunctionSelector } from '../index.js';

/**
 * A contract function Data Access Object (DAO).
 * Extends the FunctionAbi interface, adding a 'selector' property.
 * The 'selector' is a unique identifier for the function within the contract.
 */
export interface ContractFunctionDao extends FunctionAbi {
  /**
   * Unique identifier for a contract function.
   */
  selector: FunctionSelector;
}
