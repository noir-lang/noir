import { AztecAddress, EthAddress, Fr, TxRequest } from '@aztec/circuits.js';
import { Tx, TxHash } from '@aztec/tx';
import { Signature } from '../circuits.js';
import { ContractAbi } from '@aztec/noir-contracts';
import { TxReceipt } from '../tx/index.js';

export interface DeployedContract {
  abi: ContractAbi;
  address: AztecAddress;
  portalAddress: EthAddress;
}

export interface AztecRPCClient {
  addAccount(): Promise<AztecAddress>;
  getAccounts(): Promise<AztecAddress[]>;
  addContracts(contracts: DeployedContract[]): Promise<void>;
  /**
   * Is an L2 contract deployed at this address?
   * @param contractAddress - The contract data address.
   * @returns Whether the contract was deployed.
   */
  isContractDeployed(contract: AztecAddress): Promise<boolean>;
  createDeploymentTxRequest(
    abi: ContractAbi,
    args: any[],
    portalContract: EthAddress,
    contractAddressSalt: Fr,
    from: AztecAddress,
  ): Promise<TxRequest>;
  createTxRequest(functionName: string, args: any[], to: AztecAddress, from: AztecAddress): Promise<TxRequest>;
  signTxRequest(txRequest: TxRequest): Promise<Signature>;
  createTx(txRequest: TxRequest, signature: Signature): Promise<Tx>;
  sendTx(tx: Tx): Promise<TxHash>;
  getTxReceipt(txHash: TxHash): Promise<TxReceipt>;
  getStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<any>;
  // Uncomment it for milestone 1.5.
  // callTx(functionSelector: Buffer, args: Fr[], to: AztecAddress, from: AztecAddress): Promise<any>;
}
