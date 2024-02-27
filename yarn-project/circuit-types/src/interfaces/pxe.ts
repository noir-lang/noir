import { AztecAddress, CompleteAddress, Fr, GrumpkinPrivateKey, PartialAddress } from '@aztec/circuits.js';
import { ContractClassWithId, ContractInstanceWithAddress } from '@aztec/types/contracts';
import { NodeInfo } from '@aztec/types/interfaces';

import { AuthWitness } from '../auth_witness.js';
import { ContractData, ExtendedContractData } from '../contract_data.js';
import { L2Block } from '../l2_block.js';
import { L2Tx } from '../l2_tx.js';
import { GetUnencryptedLogsResponse, LogFilter } from '../logs/index.js';
import { ExtendedNote } from '../notes/index.js';
import { NoteFilter } from '../notes/note_filter.js';
import { Tx, TxHash, TxReceipt } from '../tx/index.js';
import { TxExecutionRequest } from '../tx_execution_request.js';
import { DeployedContract } from './deployed-contract.js';
import { SyncStatus } from './sync-status.js';

// docs:start:pxe-interface
/**
 * Private eXecution Environment (PXE) runs locally for each user, providing functionality for all the operations
 * needed to interact with the Aztec network, including account management, private data management,
 * transaction local simulation, and access to an Aztec node. This interface, as part of a Wallet,
 * is exposed to dapps for interacting with the network on behalf of the user.
 */
export interface PXE {
  /**
   * Insert an auth witness for a given message hash. Auth witnesses are used to authorize actions on
   * behalf of a user. For instance, a token transfer initiated by a different address may request
   * authorization from the user to move their tokens. This authorization is granted by the user
   * account contract by verifying an auth witness requested to the execution oracle. Witnesses are
   * usually a signature over a hash of the action to be authorized, but their actual contents depend
   * on the account contract that consumes them.
   *
   * @param authWitness - The auth witness to insert. Composed of an identifier, which is the hash of
   * the action to be authorized, and the actual witness as an array of fields, which are to be
   * deserialized and processed by the account contract.
   */
  addAuthWitness(authWitness: AuthWitness): Promise<void>;

  /**
   * Adding a capsule to the capsule dispenser.
   * @param capsule - An array of field elements representing the capsule.
   * @remarks A capsule is a "blob" of data that is passed to the contract through an oracle.
   */
  addCapsule(capsule: Fr[]): Promise<void>;

  /**
   * Registers a user account in PXE given its master encryption private key.
   * Once a new account is registered, the PXE Service will trial-decrypt all published notes on
   * the chain and store those that correspond to the registered account. Will do nothing if the
   * account is already registered.
   *
   * @param privKey - Private key of the corresponding user master public key.
   * @param partialAddress - The partial address of the account contract corresponding to the account being registered.
   * @returns The complete address of the account.
   */
  registerAccount(privKey: GrumpkinPrivateKey, partialAddress: PartialAddress): Promise<CompleteAddress>;

  /**
   * Registers a recipient in PXE. This is required when sending encrypted notes to
   * a user who hasn't deployed their account contract yet. Since their account is not deployed, their
   * encryption public key has not been broadcasted, so we need to manually register it on the PXE Service
   * in order to be able to encrypt data for this recipient.
   *
   * @param recipient - The complete address of the recipient
   * @remarks Called recipient because we can only send notes to this account and not receive them via this PXE Service.
   * This is because we don't have the associated private key and for this reason we can't decrypt
   * the recipient's notes. We can send notes to this account because we can encrypt them with the recipient's
   * public key.
   */
  registerRecipient(recipient: CompleteAddress): Promise<void>;

  /**
   * Retrieves the user accounts registered on this PXE Service.
   * @returns An array of the accounts registered on this PXE Service.
   */
  getRegisteredAccounts(): Promise<CompleteAddress[]>;

  /**
   * Retrieves the complete address of the account corresponding to the provided aztec address.
   * Complete addresses include the address, the partial address, and the encryption public key.
   *
   * @param address - The address of account.
   * @returns The complete address of the requested account if found.
   */
  getRegisteredAccount(address: AztecAddress): Promise<CompleteAddress | undefined>;

  /**
   * Retrieves the recipients added to this PXE Service.
   * @returns An array of recipients registered on this PXE Service.
   */
  getRecipients(): Promise<CompleteAddress[]>;

  /**
   * Retrieves the complete address of the recipient corresponding to the provided aztec address.
   * Complete addresses include the address, the partial address, and the encryption public key.
   *
   * @param address - The aztec address of the recipient.
   * @returns The complete address of the requested recipient.
   */
  getRecipient(address: AztecAddress): Promise<CompleteAddress | undefined>;

  /**
   * Adds deployed contracts to the PXE Service. Deployed contract information is used to access the
   * contract code when simulating local transactions. This is automatically called by aztec.js when
   * deploying a contract. Dapps that wish to interact with contracts already deployed should register
   * these contracts in their users' PXE Service through this method.
   *
   * @param contracts - An array of DeployedContract objects containing contract ABI, address, and portal contract.
   */
  addContracts(contracts: DeployedContract[]): Promise<void>;

  /**
   * Retrieves the addresses of contracts added to this PXE Service.
   * @returns An array of contracts addresses registered on this PXE Service.
   */
  getContracts(): Promise<AztecAddress[]>;

  /**
   * Creates a transaction based on the provided preauthenticated execution request. This will
   * run a local simulation of the private execution (and optionally of public as well), assemble
   * the zero-knowledge proof for the private execution, and return the transaction object.
   *
   * @param txRequest - An authenticated tx request ready for simulation
   * @param simulatePublic - Whether to simulate the public part of the transaction.
   * @returns A transaction ready to be sent to the network for execution.
   * @throws If the code for the functions executed in this transaction has not been made available via `addContracts`.
   */
  simulateTx(txRequest: TxExecutionRequest, simulatePublic: boolean): Promise<Tx>;

  /**
   * Sends a transaction to an Aztec node to be broadcasted to the network and mined.
   * @param tx - The transaction as created via `simulateTx`.
   * @returns A hash of the transaction, used to identify it.
   */
  sendTx(tx: Tx): Promise<TxHash>;

  /**
   * Fetches a transaction receipt for a given transaction hash. Returns a mined receipt if it was added
   * to the chain, a pending receipt if it's still in the mempool of the connected Aztec node, or a dropped
   * receipt if not found in the connected Aztec node.
   *
   * @param txHash - The transaction hash.
   * @returns A receipt of the transaction.
   */
  getTxReceipt(txHash: TxHash): Promise<TxReceipt>;

  /**
   * Fetches a transaction by its hash.
   * @param txHash - The transaction hash
   * @returns A transaction object or undefined if the transaction hasn't been mined yet
   */
  getTx(txHash: TxHash): Promise<L2Tx | undefined>;

  /**
   * Gets the storage value at the given contract storage slot.
   *
   * @remarks The storage slot here refers to the slot as it is defined in Noir not the index in the merkle tree.
   * Aztec's version of `eth_getStorageAt`.
   *
   * @param contract - Address of the contract to query.
   * @param slot - Slot to query.
   * @returns Storage value at the given contract slot.
   * @throws If the contract is not deployed.
   */
  getPublicStorageAt(contract: AztecAddress, slot: Fr): Promise<Fr>;

  /**
   * Gets notes of accounts registered in this PXE based on the provided filter.
   * @param filter - The filter to apply to the notes.
   * @returns The requested notes.
   */
  getNotes(filter: NoteFilter): Promise<ExtendedNote[]>;

  /**
   * Adds a note to the database.
   * @throws If the note hash of the note doesn't exist in the tree.
   * @param note - The note to add.
   */
  addNote(note: ExtendedNote): Promise<void>;

  /**
   * Get the given block.
   * @param number - The block number being requested.
   * @returns The blocks requested.
   */
  getBlock(number: number): Promise<L2Block | undefined>;

  /**
   * Simulate the execution of a view (read-only) function on a deployed contract without actually modifying state.
   * This is useful to inspect contract state, for example fetching a variable value or calling a getter function.
   * The function takes function name and arguments as parameters, along with the contract address
   * and optionally the sender's address.
   *
   * @param functionName - The name of the function to be called in the contract.
   * @param args - The arguments to be provided to the function.
   * @param to - The address of the contract to be called.
   * @param from - (Optional) The msg sender to set for the call.
   * @returns The result of the view function call, structured based on the function ABI.
   */
  viewTx(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress): Promise<any>;

  /**
   * Gets the extended contract data for this contract. Extended contract data includes the address,
   * portal contract address on L1, public functions, partial address, and encryption public key.
   *
   * @param contractAddress - The contract's address.
   * @returns The extended contract data if found.
   */
  getExtendedContractData(contractAddress: AztecAddress): Promise<ExtendedContractData | undefined>;

  /**
   * Gets the portal contract address on L1 for the given contract.
   *
   * @param contractAddress - The contract's address.
   * @returns The contract's portal address if found.
   */
  getContractData(contractAddress: AztecAddress): Promise<ContractData | undefined>;

  /**
   * Gets unencrypted logs based on the provided filter.
   * @param filter - The filter to apply to the logs.
   * @returns The requested logs.
   */
  getUnencryptedLogs(filter: LogFilter): Promise<GetUnencryptedLogsResponse>;

  /**
   * Fetches the current block number.
   * @returns The block number.
   */
  getBlockNumber(): Promise<number>;

  /**
   * Returns the information about the server's node. Includes current Node version, compatible Noir version,
   * L1 chain identifier, protocol version, and L1 address of the rollup contract.
   * @returns - The node information.
   */
  getNodeInfo(): Promise<NodeInfo>;

  /**
   * Checks whether all the blocks were processed (tree roots updated, txs updated with block info, etc.).
   * @returns True if there are no outstanding blocks to be synched.
   * @remarks This indicates that blocks and transactions are synched even if notes are not. Compares local block number with the block number from aztec node.
   * @deprecated Use `getSyncStatus` instead.
   */
  isGlobalStateSynchronized(): Promise<boolean>;

  /**
   * Checks if the specified account is synchronized.
   * @param account - The aztec address for which to query the sync status.
   * @returns True if the account is fully synched, false otherwise.
   * @deprecated Use `getSyncStatus` instead.
   * @remarks Checks whether all the notes from all the blocks have been processed. If it is not the case, the
   * retrieved information from contracts might be old/stale (e.g. old token balance).
   * @throws If checking a sync status of account which is not registered.
   */
  isAccountStateSynchronized(account: AztecAddress): Promise<boolean>;

  /**
   * Returns the latest block that has been synchronized globally and for each account. The global block number
   * indicates whether global state has been updated up to that block, whereas each address indicates up to which
   * block the private state has been synced for that account.
   * @returns The latest block synchronized for blocks, and the latest block synched for notes for each public key being tracked.
   */
  getSyncStatus(): Promise<SyncStatus>;

  /**
   * Returns a Contact Instance given its address, which includes the contract class identifier, portal address,
   * initialization hash, deployment salt, and public keys hash.
   * TODO(@spalladino): Should we return the public keys in plain as well here?
   * @param address - Deployment address of the contract.
   */
  getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined>;

  /**
   * Returns a Contact Class given its identifier.
   * TODO(@spalladino): The PXE actually holds artifacts and not classes, what should we return? Also,
   * should the pxe query the node for contract public info, and merge it with its own definitions?
   * @param id - Identifier of the class.
   */
  getContractClass(id: Fr): Promise<ContractClassWithId | undefined>;

  /**
   * Queries the node to check whether the contract class with the given id has been publicly registered.
   * TODO(@spalladino): This method is strictly needed to decide whether to publicly register a class or not
   * during a public deployment. We probably want a nicer and more general API for this, but it'll have to
   * do for the time being.
   * @param id - Identifier of the class.
   */
  isContractClassPubliclyRegistered(id: Fr): Promise<boolean>;
}
// docs:end:pxe-interface
