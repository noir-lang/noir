import { AztecAddress, EthAddress, Fr, PartialContractAddress, PrivateKey, PublicKey } from '@aztec/circuits.js';
import { ContractAbi } from '@aztec/foundation/abi';
import {
  ContractData,
  ContractPublicData,
  L2BlockL2Logs,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

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
 * Represents an Aztec RPC implementation.
 * Provides functionality for all the operations needed to interact with the Aztec network,
 * including account management, contract deployment, transaction creation, and execution,
 * as well as storage and view functions for smart contracts.
 */
export interface AztecRPC {
  /**
   * Registers an account backed by an account contract.
   *
   * @param privKey - Private key of the corresponding user master public key.
   * @param address - Address of the account contract.
   * @param partialContractAddress - The partially computed address of the account contract.
   * @returns The address of the account contract.
   */
  addAccount(
    privKey: PrivateKey,
    address: AztecAddress,
    partialContractAddress: PartialContractAddress,
  ): Promise<AztecAddress>;

  /**
   * Retrieves the list of Aztec addresses added to this rpc server
   * The addresses are returned as a promise that resolves to an array of AztecAddress objects.
   *
   * @returns A promise that resolves to an array of AztecAddress instances.
   */
  getAccounts(): Promise<AztecAddress[]>;

  /**
   * Retrieve the public key associated with an address.
   * Throws an error if the account is not found in the key store.
   *
   * @param address - The AztecAddress instance representing the account to get public key for.
   * @returns A Promise resolving to the PublicKey instance representing the public key.
   */
  getPublicKey(address: AztecAddress): Promise<PublicKey>;

  /**
   * Add an array of deployed contracts to the database.
   * Each contract should contain ABI, address, and portalContract information.
   *
   * @param contracts - An array of DeployedContract objects containing contract ABI, address, and portalContract.
   * @returns A Promise that resolves once all the contracts have been added to the database.
   */
  addContracts(contracts: DeployedContract[]): Promise<void>;

  /**
   * Is an L2 contract deployed at this address?
   * @param contract - The contract data address.
   * @returns Whether the contract was deployed.
   */
  isContractDeployed(contract: AztecAddress): Promise<boolean>;

  /**
   * Create a transaction for a contract function call with the provided arguments.
   * Throws an error if the contract or function is unknown.
   *
   * @param txRequest - An authenticated tx request ready for simulation
   * @param optionalFromAddress - The address to simulate from
   * @returns A Tx ready to send to the p2p pool for execution.
   */
  simulateTx(txRequest: TxExecutionRequest): Promise<Tx>;

  /**
   * Send a transaction.
   * @param tx - The transaction.
   * @returns A hash of the transaction, used to identify it.
   */
  sendTx(tx: Tx): Promise<TxHash>;

  /**
   * Fetches a transaction receipt for a tx.
   * @param txHash - The transaction hash.
   * @returns A receipt of the transaction.
   */
  getTxReceipt(txHash: TxHash): Promise<TxReceipt>;

  /**
   * Retrieves the preimage data at a specified contract address and storage slot.
   * The returned data is an array of note preimage items, with each item containing its value.
   *
   * @param contract - The AztecAddress of the target contract.
   * @param storageSlot - The Fr representing the storage slot to be fetched.
   * @returns A promise that resolves to an array of note preimage items, each containing its value.
   */
  getPreimagesAt(contract: AztecAddress, storageSlot: Fr): Promise<bigint[][]>;

  /**
   * Retrieves the public storage data at a specified contract address and storage slot.
   * The returned data is data at the storage slot or throws an error if the contract is not deployed.
   *
   * @param contract - The AztecAddress of the target contract.
   * @param storageSlot - The Fr representing the storage slot to be fetched.
   * @returns A buffer containing the public storage data at the storage slot.
   */
  getPublicStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<Buffer | undefined>;

  /**
   * Simulate the execution of a view (read-only) function on a deployed contract without actually modifying state.
   * This is useful to inspect contract state, for example fetching a variable value or calling a getter function.
   * The function takes function name and arguments as parameters, along with the contract address
   * and optionally the sender's address.
   *
   * @param functionName - The name of the function to be called in the contract.
   * @param args - The arguments to be provided to the function.
   * @param to - The address of the contract to be called.
   * @param from - (Optional) The caller of the transaction.
   * @returns The result of the view function call, structured based on the function ABI.
   */
  viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress): Promise<any>;

  /**
   * Lookup the L2 contract data for this contract.
   * Contains the ethereum portal address and bytecode.
   * @param contractAddress - The contract data address.
   * @returns The complete contract data including portal address & bytecode (if we didn't throw an error).
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractPublicData | undefined>;

  /**
   * Lookup the L2 contract info for this contract.
   * Contains the ethereum portal address .
   * @param contractAddress - The contract data address.
   * @returns The contract's address & portal address.
   */
  getContractInfo(contractAddress: AztecAddress): Promise<ContractData | undefined>;

  /**
   * Gets L2 block unencrypted logs.
   * @param from - Number of the L2 block to which corresponds the first unencrypted logs to be returned.
   * @param limit - The maximum number of unencrypted logs to return.
   * @returns The requested unencrypted logs.
   */
  getUnencryptedLogs(from: number, limit: number): Promise<L2BlockL2Logs[]>;

  /**
   * Get latest L2 block number.
   * @returns The latest block number.
   */
  getBlockNum(): Promise<number>;

  /**
   * Returns the information about the server's node
   * @returns - The node information.
   */
  getNodeInfo(): Promise<NodeInfo>;

  /**
   * Adds public key and partial address to a database.
   * @param address - Address of the account to add public key and partial address for.
   * @param publicKey - Public key of the corresponding user.
   * @param partialAddress - The partially computed address of the account contract.
   * @returns A Promise that resolves once the public key has been added to the database.
   */
  addPublicKeyAndPartialAddress(
    address: AztecAddress,
    publicKey: PublicKey,
    partialAddress: PartialContractAddress,
  ): Promise<void>;

  /**
   * Retrieve the public key and partial contract address associated with an address.
   * Throws an error if the account is not found in the key store.
   *
   * @param address - The AztecAddress instance representing the account to get public key and partial address for.
   * @returns A Promise resolving to the PublicKey instance representing the public key.
   */
  getPublicKeyAndPartialAddress(address: AztecAddress): Promise<[PublicKey, PartialContractAddress]>;

  /**
   * Return true if the top level block synchronisation is up to date
   * This indicates that blocks and transactions are synched even if notes are not
   * @returns True if there are no outstanding blocks to be synched
   */
  isSynchronised(): Promise<boolean>;

  /**
   * Returns true if the account specified by the given address is synched to the latest block
   * @param account - The aztec address for which to query the sync status
   * @returns True if the account is fully synched, false otherwise
   */
  isAccountSynchronised(account: AztecAddress): Promise<boolean>;
}
