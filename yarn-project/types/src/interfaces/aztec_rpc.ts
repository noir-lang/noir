import { AztecAddress, EthAddress, Fr, GrumpkinPrivateKey, PartialAddress } from '@aztec/circuits.js';
import { ContractAbi } from '@aztec/foundation/abi';
import {
  CompleteAddress,
  ContractData,
  ExtendedContractData,
  L2BlockL2Logs,
  NotePreimage,
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
   * The complete address representing the contract on L2.
   */
  completeAddress: CompleteAddress;
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
  /**
   * Identifier of the client software.
   */
  client: string;
};

/** Provides up to which block has been synced by different components. */
export type SyncStatus = {
  /** Up to which block has been synched for blocks and txs. */
  blocks: number;
  /** Up to which block has been synched for notes, indexed by each public key being monitored. */
  notes: Record<string, number>;
};

// docs:start:rpc-interface
/**
 * Represents an Aztec RPC implementation.
 * Provides functionality for all the operations needed to interact with the Aztec network,
 * including account management, contract deployment, transaction creation, and execution,
 * as well as storage and view functions for smart contracts.
 */
export interface AztecRPC {
  /**
   * Insert a witness for a given message hash.
   * @param messageHash - The message hash to insert witness at
   * @param witness - The witness to insert
   */
  addAuthWitness(messageHash: Fr, witness: Fr[]): Promise<void>;

  /**
   * Registers an account in the Aztec RPC server.
   *
   * @param privKey - Private key of the corresponding user master public key.
   * @param partialAddress - A partial address of the account contract corresponding to the account being registered.
   * @returns Empty promise.
   * @throws If the account is already registered.
   */
  registerAccount(privKey: GrumpkinPrivateKey, partialAddress: PartialAddress): Promise<void>;

  /**
   * Registers recipient in the Aztec RPC server.
   * @param recipient - A complete address of the recipient
   * @returns Empty promise.
   * @remarks Called recipient because we can only send notes to this account and not receive them via this RPC server.
   *          This is because we don't have the associated private key and for this reason we can't decrypt
   *          the recipient's notes. We can send notes to this account because we can encrypt them with the recipient's
   *          public key.
   */
  registerRecipient(recipient: CompleteAddress): Promise<void>;

  /**
   * Retrieves the list of accounts added to this rpc server.
   * The addresses are returned as a promise that resolves to an array of CompleteAddress objects.
   *
   * @returns A promise that resolves to an array of the accounts registered on this RPC server.
   */
  getAccounts(): Promise<CompleteAddress[]>;

  /**
   * Retrieves the complete address of the account corresponding to the provided aztec address.
   * @param address - The aztec address of the account contract.
   * @returns A promise that resolves to the complete address of the requested account.
   */
  getAccount(address: AztecAddress): Promise<CompleteAddress | undefined>;

  /**
   * Retrieves the list of recipients added to this rpc server.
   * The addresses are returned as a promise that resolves to an array of CompleteAddress objects.
   *
   * @returns A promise that resolves to an array registered recipients on this RPC server.
   */
  getRecipients(): Promise<CompleteAddress[]>;

  /**
   * Retrieves the complete address of the recipient corresponding to the provided aztec address.
   * @param address - The aztec address of the recipient.
   * @returns A promise that resolves to the complete address of the requested recipient.
   */
  getRecipient(address: AztecAddress): Promise<CompleteAddress | undefined>;

  /**
   * Add an array of deployed contracts to the database.
   * Each contract should contain ABI, address, and portalContract information.
   *
   * @param contracts - An array of DeployedContract objects containing contract ABI, address, and portalContract.
   * @returns A Promise that resolves once all the contracts have been added to the database.
   */
  addContracts(contracts: DeployedContract[]): Promise<void>;

  /**
   * Retrieves the list of addresses of contracts added to this rpc server.
   * @returns A promise that resolves to an array of contracts addresses registered on this RPC server.
   */
  getContracts(): Promise<AztecAddress[]>;

  /**
   * Create a transaction for a contract function call with the provided arguments.
   * Throws an error if the contract or function is unknown.
   *
   * @param txRequest - An authenticated tx request ready for simulation
   * @param simulatePublic - Whether to simulate the public part of the transaction.
   * @returns A Tx ready to send to the p2p pool for execution.
   */
  simulateTx(txRequest: TxExecutionRequest, simulatePublic: boolean): Promise<Tx>;

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
   * Retrieves the private storage data at a specified contract address and storage slot.
   * The returned data is data at the storage slot or throws an error if the contract is not deployed.
   *
   * @param owner - The address for whom the private data is encrypted.
   * @param contract - The AztecAddress of the target contract.
   * @param storageSlot - The Fr representing the storage slot to be fetched.
   * @returns A set of note preimages for the owner in that contract and slot.
   */
  getPrivateStorageAt(owner: AztecAddress, contract: AztecAddress, storageSlot: Fr): Promise<NotePreimage[]>;

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
   * Get the extended contract data for this contract.
   * @param contractAddress - The contract data address.
   * @returns The extended contract data or undefined if not found.
   */
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined>;

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
   * Fetches the current block number.
   * @returns The block number.
   */
  getBlockNumber(): Promise<number>;

  /**
   * Returns the information about the server's node
   * @returns - The node information.
   */
  getNodeInfo(): Promise<NodeInfo>;

  /**
   * Checks whether all the blocks were processed (tree roots updated, txs updated with block info, etc.).
   * @returns True if there are no outstanding blocks to be synched.
   * @remarks This indicates that blocks and transactions are synched even if notes are not.
   * @remarks Compares local block number with the block number from aztec node.
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
// docs:end:rpc-interface
