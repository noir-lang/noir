import { AztecAddress, EthAddress } from '@aztec/circuits.js';
import { ContractAbi } from '../noir.js';
import { ContractDao } from './contract_dao.js';

export interface ContractDataSource {
  addContract(address: AztecAddress, portalAddress: EthAddress, abi: ContractAbi, deployed?: boolean): Promise<void>;
  getContract(address: AztecAddress): Promise<ContractDao | undefined>;
}
