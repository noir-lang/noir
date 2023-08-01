import { ContractFunctionDao } from '@aztec/circuits.js';
import { ContractAbi, FunctionType, generateFunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';

import { EncodedContractFunction } from './contract_data.js';

/**
 * A contract Data Access Object (DAO).
 * Contains the contract's address, portal contract address, and an array of ContractFunctionDao objects.
 * Each ContractFunctionDao object includes FunctionAbi data and the function selector buffer.
 */
export interface ContractDao extends ContractAbi {
  /**
   * The noir contract address.
   */
  address: AztecAddress;
  /**
   * The Ethereum address of the L1 contract serving as a bridge for cross-layer interactions.
   */
  portalContract: EthAddress;
  /**
   * An array of contract functions with additional selector property.
   */
  functions: ContractFunctionDao[];
}

/**
 * Converts the given contract ABI into a ContractDao object that includes additional properties
 * such as the address, portal contract, and function selectors.
 *
 * @param abi - The contract ABI.
 * @param address - The AztecAddress representing the contract's address.
 * @param portalContract - The EthAddress representing the address of the associated portal contract.
 * @returns A ContractDao object containing the provided information along with generated function selectors.
 */
export function toContractDao(abi: ContractAbi, address: AztecAddress, portalContract: EthAddress): ContractDao {
  const functions = abi.functions.map(f => ({
    ...f,
    selector: generateFunctionSelector(f.name, f.parameters),
  }));
  return {
    ...abi,
    address,
    functions,
    portalContract,
  };
}

/**
 * Return public functions from the newly deployed contract to be injected into the tx object.
 * @param newContract - The new contract
 * @returns List of EncodedContractFunction.
 */
export function getNewContractPublicFunctions(newContract: ContractDao) {
  return newContract.functions
    .filter(c => c.functionType === FunctionType.OPEN)
    .map(
      fn =>
        new EncodedContractFunction(
          generateFunctionSelector(fn.name, fn.parameters),
          fn.isInternal ?? false,
          Buffer.from(fn.bytecode, 'base64'),
        ),
    );
}
