import { AztecAddress, EthAddress, Fr, PartialAddress, PrivateKey, PublicKey } from '@aztec/circuits.js';
import { ContractAbi } from '@aztec/foundation/abi';
import {
  ContractData,
  ContractDataAndBytecode,
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
  /**
   * The rollup contract address
   */
  rollupAddress: EthAddress;
};

/** Provides up to which block has been synced by different components. */
export type SyncStatus = {
  /** Up to which block has been synched for blocks and txs. */
  blocks: number;
  /** Up to which block has been synched for notes, indexed by each public key being monitored. */
  notes: Record<string, number>;
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
   * @param partialAddress - The partially computed address of the account contract.
   * @returns The address of the account contract.
   */
  addAccount(privKey: PrivateKey, address: AztecAddress, partialAddress: PartialAddress): Promise<AztecAddress>;

  /**
   * Retrieves the list of Aztec addresses added to this rpc server
   * The addresses are returned as a promise that resolves to an array of AztecAddress objects.
   *
   * @returns A promise that resolves to an array of AztecAddress instances.
   */
  getAccounts(): Promise<AztecAddress[]>;

  /**
   * Add an array of deployed contracts to the database.
   * Each contract should contain ABI, address, and portalContract information.
   *
   * @param contracts - An array of DeployedContract objects containing contract ABI, address, and portalContract.
   * @returns A Promise that resolves once all the contracts have been added to the database.
   */
  addContracts(contracts: DeployedContract[]): Promise<void>;

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
  getContractDataAndBytecode(contractAddress: AztecAddress): Promise<ContractDataAndBytecode | undefined>;

  /**
   * Lookup the L2 contract data for this contract.
   * Contains the ethereum portal address .
   * @param contractAddress - The contract data address.
   * @returns The contract's address & portal address.
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined>;

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
    partialAddress: PartialAddress,
  ): Promise<void>;

  /**
   * Retrieve the public key and partial address associated with an address.
   * Throws an error if the account is not found in the key store.
   *
   * @param address - The AztecAddress instance representing the account to get public key and partial address for.
   * @returns A Promise resolving to the PublicKey instance representing the public key.
   * @remarks The public key and partial address form a preimage of a contract address. See
   * https://github.com/AztecProtocol/aztec-packages/blob/janb/rpc-interface-cleanup/docs/docs/concepts/foundation/accounts/keys.md#addresses-partial-addresses-and-public-keys
   */
  getPublicKeyAndPartialAddress(address: AztecAddress): Promise<[PublicKey, PartialAddress]>;

  /**
   * Checks whether all the blocks were processed (tree roots updated, txs updated with block info, etc.).
   * @returns True if there are no outstanding blocks to be synched.
   * @remarks This indicates that blocks and transactions are synched even if notes are not.
   * @remarks Compares local block height with the block height from aztec node.
   */
  isGlobalStateSynchronised(): Promise<boolean>;

  /**
   * Checks if the specified account is synchronised.
   * @param account - The aztec address for which to query the sync status.
   * @returns True if the account is fully synched, false otherwise.
   * @remarks Checks whether all the notes from all the blocks have been processed. If it is not the case, the
   *          retrieved information from contracts might be old/stale (e.g. old token balance).
   */
  isAccountStateSynchronised(account: AztecAddress): Promise<boolean>;

  /**
   * Returns the latest block that has been synchronised by the synchronizer and each account.
   * @returns The latest block synchronised for blocks, and the latest block synched for notes for each public key being tracked.
   */
  getSyncStatus(): Promise<SyncStatus>;
}
