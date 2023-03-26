import { generateFunctionSelector } from '../abi_coder/index.js';
import { AztecAddress, EthAddress } from '../circuits.js';
import { ContractAbi, FunctionAbi } from '../noir.js';

export interface ContractFunctionDao extends FunctionAbi {
  selector: Buffer;
}

export interface ContractDao extends ContractAbi {
  address: AztecAddress;
  portalAddress: EthAddress;
  functions: ContractFunctionDao[];
  deployed: boolean;
}

export function functionAbiToFunctionDao(abi: FunctionAbi) {
  const selector = generateFunctionSelector(abi.name, abi.parameters);
  return {
    ...abi,
    selector,
  };
}

export function contractAbiToContractDao(
  address: AztecAddress,
  portalAddress: EthAddress,
  abi: ContractAbi,
  deployed: boolean,
): ContractDao {
  return {
    address,
    portalAddress,
    functions: abi.functions.map(functionAbiToFunctionDao),
    deployed,
  };
}
