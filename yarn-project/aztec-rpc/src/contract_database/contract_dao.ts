import { generateFunctionSelector } from '../abi_coder/index.js';
import { AztecAddress, EthAddress } from '@aztec/circuits.js';
import { ContractAbi, FunctionAbi } from '../noir.js';

export interface ContractFunctionDao extends FunctionAbi {
  selector: Buffer;
}

export interface ContractDao extends ContractAbi {
  address: AztecAddress;
  portalAddress: EthAddress;
  functions: ContractFunctionDao[];
}

export function functionAbiToFunctionDao(abi: FunctionAbi) {
  return {
    ...abi,
    selector: generateFunctionSelector(abi.name, abi.parameters),
  };
}

export function contractAbiToContractDao(
  address: AztecAddress,
  portalAddress: EthAddress,
  abi: ContractAbi,
): ContractDao {
  return {
    address,
    portalAddress,
    functions: abi.functions.map(functionAbiToFunctionDao),
  };
}
