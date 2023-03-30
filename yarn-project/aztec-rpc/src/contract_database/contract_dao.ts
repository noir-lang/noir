import { AztecAddress, EthAddress } from '@aztec/circuits.js';
import { ContractAbi, FunctionAbi } from '@aztec/noir-contracts';

export interface ContractFunctionDao extends FunctionAbi {
  selector: Buffer;
}

export interface ContractDao extends ContractAbi {
  address: AztecAddress;
  portalAddress: EthAddress;
  functions: ContractFunctionDao[];
}
