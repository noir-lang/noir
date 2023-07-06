import { AztecAddress, EthAddress, Fr } from '@aztec/circuits.js';
import { ContractAbi } from '@aztec/foundation/abi';
import { Point } from '@aztec/foundation/fields';
import { PublicKey } from '@aztec/key-store';
import {
  ContractData,
  ContractPublicData,
  L2BlockL2Logs,
  PartialContractAddress,
  Tx,
  TxExecutionRequest,
  TxHash,
} from '@aztec/types';
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
 * Provides basic information about the running node.
 */
export type NodeInfo = {
  /**
   * The version number of the node.
   */
  version: number;
  /**
   * The network's chain id.
   */
  chainId: number;
};

/**
 * Represents the data generated as part of contract deployment.
 */
export type DeploymentInfo = {
  /**
   * The derived aztec address of the contract.
   */
  address: AztecAddress;
  /**
   * The partially derived aztec address of the contract.
   */
  partialAddress: PartialContractAddress;
  /**
   * The contract's constructor hash.
   */
  constructorHash: Fr;
  /**
   * The root of the contract's function tree.
   */
  functionTreeRoot: Fr;
  /**
   * The public key associated with the contract.
   */
  publicKey: PublicKey;
};

/**
 * Represents an Aztec RPC implementation.
 * Provides functionality for all the operations needed to interact with the Aztec network,
 * including account management, contract deployment, transaction creation, and execution,
 * as well as storage and view functions for smart contracts.
 */
export interface AztecRPC {
  addAccount(
    privKey: Buffer,
    address: AztecAddress,
    partialContractAddress: PartialContractAddress,
    abi?: ContractAbi,
  ): Promise<AztecAddress>;
  getAccounts(): Promise<AztecAddress[]>;
  getAccountPublicKey(address: AztecAddress): Promise<Point>;
  addContracts(contracts: DeployedContract[]): Promise<void>;
  /**
   * Is an L2 contract deployed at this address?
   * @param contractAddress - The contract data address.
   * @returns Whether the contract was deployed.
   */
  isContractDeployed(contract: AztecAddress): Promise<boolean>;
  simulateTx(txRequest: TxExecutionRequest, optionalFromAddress?: AztecAddress): Promise<Tx>;
  sendTx(tx: Tx): Promise<TxHash>;
  getTxReceipt(txHash: TxHash): Promise<TxReceipt>;
  getStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<any>;
  viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress): Promise<any>;
  getContractData(contractAddress: AztecAddress): Promise<ContractPublicData | undefined>;
  getContractInfo(contractAddress: AztecAddress): Promise<ContractData | undefined>;
  getUnencryptedLogs(from: number, take: number): Promise<L2BlockL2Logs[]>;
  getBlockNum(): Promise<number>;
  getNodeInfo(): Promise<NodeInfo>;
}
