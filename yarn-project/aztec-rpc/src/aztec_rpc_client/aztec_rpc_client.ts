import { AztecAddress, EthAddress, Fr, TxRequest, EcdsaSignature } from '@aztec/circuits.js';
import { Tx, TxHash } from '@aztec/tx';
import { ContractAbi } from '@aztec/noir-contracts';
import { TxReceipt } from '../tx/index.js';
import { Point } from '@aztec/foundation';

export interface DeployedContract {
  abi: ContractAbi;
  address: AztecAddress;
  portalContract: EthAddress;
}

export interface AztecRPCClient {
  addAccount(): Promise<AztecAddress>;
  getAccounts(): Promise<AztecAddress[]>;
  getAccountPublicKey(address: AztecAddress): Promise<Point>;
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
  signTxRequest(txRequest: TxRequest): Promise<EcdsaSignature>;
  createTx(txRequest: TxRequest, signature: EcdsaSignature): Promise<Tx>;
  sendTx(tx: Tx): Promise<TxHash>;
  getTxReceipt(txHash: TxHash): Promise<TxReceipt>;
  getStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<any>;
  viewTx(functionName: string, args: any[], to: AztecAddress, from: AztecAddress): Promise<any>;
}
