import { AztecAddress, EthAddress } from '@aztec/circuits.js';
import { ContractAbi, FunctionAbi } from '@aztec/noir-contracts';
import { generateFunctionSelector } from '../abi_coder/index.js';

export interface ContractFunctionDao extends FunctionAbi {
  selector: Buffer;
}

export interface ContractDao extends ContractAbi {
  address: AztecAddress;
  portalContract: EthAddress;
  functions: ContractFunctionDao[];
}

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
