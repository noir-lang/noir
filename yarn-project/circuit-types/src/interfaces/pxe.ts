import {
  type AztecAddress,
  type CompleteAddress,
  type Fq,
  type Fr,
  type PartialAddress,
  type Point,
} from '@aztec/circuits.js';
import { type ContractArtifact, type EventSelector } from '@aztec/foundation/abi';
import {
  type ContractClassWithId,
  type ContractInstanceWithAddress,
  type ProtocolContractAddresses,
} from '@aztec/types/contracts';
import { type NodeInfo } from '@aztec/types/interfaces';

import { type AuthWitness } from '../auth_witness.js';
import { type L2Block } from '../l2_block.js';
import { type GetUnencryptedLogsResponse, type L1EventPayload, type LogFilter } from '../logs/index.js';
import { type IncomingNotesFilter } from '../notes/incoming_notes_filter.js';
import { type ExtendedNote, type OutgoingNotesFilter } from '../notes/index.js';
import { type NoteProcessorStats } from '../stats/stats.js';
import { type SimulatedTx, type Tx, type TxHash, type TxReceipt } from '../tx/index.js';
import { type TxEffect } from '../tx_effect.js';
import { type TxExecutionRequest } from '../tx_execution_request.js';
import { type SyncStatus } from './sync-status.js';

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
   * Fetches the serialized auth witness for a given message hash or returns undefined if not found.
   * @param messageHash - The hash of the message for which to get the auth witness.
   * @returns The serialized auth witness for the given message hash.
   */
  getAuthWitness(messageHash: Fr): Promise<Fr[] | undefined>;

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
   * @param secretKey - Secret key of the corresponding user master public key.
   * @param partialAddress - The partial address of the account contract corresponding to the account being registered.
   * @returns The complete address of the account.
   */
  registerAccount(secretKey: Fr, partialAddress: PartialAddress): Promise<CompleteAddress>;

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
   * Rotates master nullifier keys.
   * @param address - The address of the account we want to rotate our key for.
   * @param newNskM - The new master nullifier secret key we want to use.
   * @remarks - One should not use this function directly without also calling the canonical key registry to rotate
   * the new master nullifier secret key's derived master nullifier public key.
   * Therefore, it is preferred to use rotateNullifierKeys on AccountWallet, as that handles the call to the Key Registry as well.
   *
   * This does not hinder our ability to spend notes tied to a previous master nullifier public key, provided we have the master nullifier secret key for it.
   */
  rotateNskM(address: AztecAddress, newNskM: Fq): Promise<void>;

  /**
   * Registers a contract class in the PXE without registering any associated contract instance with it.
   *
   * @param artifact - The build artifact for the contract class.
   */
  registerContractClass(artifact: ContractArtifact): Promise<void>;

  /**
   * Adds deployed contracts to the PXE Service. Deployed contract information is used to access the
   * contract code when simulating local transactions. This is automatically called by aztec.js when
   * deploying a contract. Dapps that wish to interact with contracts already deployed should register
   * these contracts in their users' PXE Service through this method.
   *
   * @param contract - A contract instance to register, with an optional artifact which can be omitted if the contract class has already been registered.
   */
  registerContract(contract: { instance: ContractInstanceWithAddress; artifact?: ContractArtifact }): Promise<void>;

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
   * Also throws if simulatePublic is true and public simulation reverts.
   */
  proveTx(txRequest: TxExecutionRequest, simulatePublic: boolean): Promise<Tx>;

  /**
   * Simulates a transaction based on the provided preauthenticated execution request.
   * This will run a local simulation of private execution (and optionally of public as well), assemble
   * the zero-knowledge proof for the private execution, and return the transaction object along
   * with simulation results (return values).
   *
   *
   * Note that this is used with `ContractFunctionInteraction::simulateTx` to bypass certain checks.
   * In that case, the transaction returned is only potentially ready to be sent to the network for execution.
   *
   *
   * @param txRequest - An authenticated tx request ready for simulation
   * @param simulatePublic - Whether to simulate the public part of the transaction.
   * @param msgSender - (Optional) The message sender to use for the simulation.
   * @returns A simulated transaction object that includes a transaction that is potentially ready
   * to be sent to the network for execution, along with public and private return values.
   * @throws If the code for the functions executed in this transaction has not been made available via `addContracts`.
   * Also throws if simulatePublic is true and public simulation reverts.
   */
  simulateTx(txRequest: TxExecutionRequest, simulatePublic: boolean, msgSender?: AztecAddress): Promise<SimulatedTx>;

  /**
   * Sends a transaction to an Aztec node to be broadcasted to the network and mined.
   * @param tx - The transaction as created via `proveTx`.
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
   * Get a tx effect.
   * @param txHash - The hash of a transaction which resulted in the returned tx effect.
   * @returns The requested tx effect.
   */
  getTxEffect(txHash: TxHash): Promise<TxEffect | undefined>;

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
   * Gets incoming notes of accounts registered in this PXE based on the provided filter.
   * @param filter - The filter to apply to the notes.
   * @returns The requested notes.
   */
  getIncomingNotes(filter: IncomingNotesFilter): Promise<ExtendedNote[]>;

  /**
   * Gets outgoing notes of accounts registered in this PXE based on the provided filter.
   * @param filter - The filter to apply to the notes.
   * @returns The requested notes.
   */
  getOutgoingNotes(filter: OutgoingNotesFilter): Promise<ExtendedNote[]>;

  /**
   * Finds the nonce(s) for a given note.
   * @param note - The note to find the nonces for.
   * @returns The nonces of the note.
   * @remarks More than a single nonce may be returned since there might be more than one nonce for a given note.
   * TODO(#4956): Un-expose this
   */
  getNoteNonces(note: ExtendedNote): Promise<Fr[]>;

  /**
   * Adds a note to the database.
   * @throws If the note hash of the note doesn't exist in the tree.
   * @param note - The note to add.
   */
  addNote(note: ExtendedNote): Promise<void>;

  /**
   * Adds a nullified note to the database.
   * @throws If the note hash of the note doesn't exist in the tree.
   * @param note - The note to add.
   * @dev We are not deriving a nullifier in this function since that would require having the nullifier secret key
   * which is undesirable. Instead, we are just adding the note to the database as nullified and the nullifier is set
   * to 0 in the db.
   */
  addNullifiedNote(note: ExtendedNote): Promise<void>;

  /**
   * Get the given block.
   * @param number - The block number being requested.
   * @returns The blocks requested.
   */
  getBlock(number: number): Promise<L2Block | undefined>;

  /**
   * Simulate the execution of an unconstrained function on a deployed contract without actually modifying state.
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
  simulateUnconstrained(functionName: string, args: any[], to: AztecAddress, from?: AztecAddress): Promise<any>;

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
   * Fetches the current proven block number.
   * @returns The block number.
   */
  getProvenBlockNumber(): Promise<number>;

  /**
   * Returns the information about the server's node. Includes current Node version, compatible Noir version,
   * L1 chain identifier, protocol version, and L1 address of the rollup contract.
   * @returns - The node information.
   */
  getNodeInfo(): Promise<NodeInfo>;

  /**
   * Returns information about this PXE.
   */
  getPXEInfo(): Promise<PXEInfo>;

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
   * Returns the note processor stats.
   * @returns The note processor stats for notes for each public key being tracked.
   */
  getSyncStats(): Promise<{ [key: string]: NoteProcessorStats }>;

  /**
   * Returns a Contact Instance given its address, which includes the contract class identifier,
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
   * Returns the contract artifact associated to a contract class.
   * @param id - Identifier of the class.
   */
  getContractArtifact(id: Fr): Promise<ContractArtifact | undefined>;

  /**
   * Queries the node to check whether the contract class with the given id has been publicly registered.
   * TODO(@spalladino): This method is strictly needed to decide whether to publicly register a class or not
   * during a public deployment. We probably want a nicer and more general API for this, but it'll have to
   * do for the time being.
   * @param id - Identifier of the class.
   */
  isContractClassPubliclyRegistered(id: Fr): Promise<boolean>;

  /**
   * Queries the node to check whether the contract instance with the given address has been publicly deployed,
   * regardless of whether this PXE knows about the contract or not.
   * TODO(@spalladino): Same notes as above.
   */
  isContractPubliclyDeployed(address: AztecAddress): Promise<boolean>;

  /**
   * Queries the node to check whether the contract instance with the given address has been initialized,
   * by checking the standard initialization nullifier.
   * @param address - Address of the contract to check.
   */
  isContractInitialized(address: AztecAddress): Promise<boolean>;

  /**
   * Returns the events of a specified type given search parameters.
   * @param type - The type of the event to search for—Encrypted, or Unencrypted.
   * @param eventMetadata - Identifier of the event. This should be the class generated from the contract. e.g. Contract.events.Event
   * @param from - The block number to search from.
   * @param limit - The amount of blocks to search.
   * @param vpks - (Used for encrypted logs only) The viewing (incoming and outgoing) public keys that correspond to the viewing secret keys that can decrypt the log.
   * @returns - The deserialized events.
   */
  getEvents<T>(
    type: EventType,
    eventMetadata: EventMetadata<T>,
    from: number,
    limit: number,
    vpks: Point[],
  ): Promise<T[]>;
}
// docs:end:pxe-interface

/**
 * The shape of the event generated on the Contract.
 */
export interface EventMetadata<T> {
  decode(payload: L1EventPayload): T | undefined;
  eventSelector: EventSelector;
  fieldNames: string[];
}

/**
 * This is used in getting events via the filter
 */
export enum EventType {
  Encrypted = 'Encrypted',
  Unencrypted = 'Unencrypted',
}

/**
 * Provides basic information about the running PXE.
 */
export interface PXEInfo {
  /**
   * Version as tracked in the aztec-packages repository.
   */
  pxeVersion: string;
  /**
   * Protocol contract addresses
   */
  protocolContractAddresses: ProtocolContractAddresses;
}
