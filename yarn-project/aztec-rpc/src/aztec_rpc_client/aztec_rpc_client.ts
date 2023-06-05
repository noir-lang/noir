import { AztecAddress, EcdsaSignature, EthAddress, Fr } from '@aztec/circuits.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { Point } from '@aztec/foundation/fields';
import { Tx, TxExecutionRequest, TxHash } from '@aztec/types';
import { TxReceipt } from '../tx/index.js';

/**
 * Represents a deployed contract on the Aztec network.
 * Contains the contract ABI, address, and associated portal contract address.
 */
export interface DeployedContract {
  /**
   * The Application Binary Interface of the deployed contract.
   */
  abi: ContractAbi;
  /**
   * The address representing the contract on L2.
   */
  address: AztecAddress;
  /**
   * The Ethereum address of the L1 portal contract.
   */
  portalContract: EthAddress;
}

/**
 * Represents an Aztec RPC client implementation.
 * Provides functionality for all the operations needed to interact with the Aztec network,
 * including account management, contract deployment, transaction creation, and execution,
 * as well as storage and view functions for smart contracts.
 */
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
    contractAddressSalt?: Fr,
    from?: AztecAddress,
  ): Promise<TxExecutionRequest>;
  createTxRequest(
    functionName: string,
    args: any[],
    to: AztecAddress,
    from?: AztecAddress,
  ): Promise<TxExecutionRequest>;
  signTxRequest(txRequest: TxExecutionRequest): Promise<EcdsaSignature>;
  createTx(txRequest: TxExecutionRequest, signature: EcdsaSignature): Promise<Tx>;
  sendTx(tx: Tx): Promise<TxHash>;
  getTxReceipt(txHash: TxHash): Promise<TxReceipt>;
  getStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<any>;
  viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress): Promise<any>;
}
