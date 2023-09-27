import { AztecAddress, Fr, GrumpkinPrivateKey, PartialAddress } from '@aztec/circuits.js';
import {
  AuthWitness,
  CompleteAddress,
  ContractData,
  ExtendedContractData,
  L2BlockL2Logs,
  L2Tx,
  NotePreimage,
  PublicKey,
  Tx,
  TxExecutionRequest,
  TxHash,
  TxReceipt,
} from '@aztec/types';

import { DeployedContract } from './deployed-contract.js';
import { NodeInfo } from './node-info.js';
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
   * Insert an auth witness for a given message hash. Auth witnesses are used to authorise actions on
   * behalf of a user. For instance, a token transfer initiated by a different address may request
   * authorisation from the user to move their tokens. This authorisation is granted by the user
   * account contract by verifying an auth witness requested to the execution oracle. Witnesses are
   * usually a signature over a hash of the action to be authorised, but their actual contents depend
   * on the account contract that consumes them.
   *
   * @param authWitness - The auth witness to insert. Composed of an identifier, which is the hash of
   * the action to be authorised, and the actual witness as an array of fields, which are to be
   * deserialized and processed by the account contract.
   */
  addAuthWitness(authWitness: AuthWitness): Promise<void>;

  /**
   * Registers a user account in PXE given its master encryption private key.
   * Once a new account is registered, the PXE Service will trial-decrypt all published notes on
   * the chain and store those that correspond to the registered account.
   *
   * @param privKey - Private key of the corresponding user master public key.
   * @param partialAddress - The partial address of the account contract corresponding to the account being registered.
   * @throws If the account is already registered.
   */
  registerAccount(privKey: GrumpkinPrivateKey, partialAddress: PartialAddress): Promise<void>;

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
   * @returns A transaction ready to be sent to the network for excution.
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
   * Retrieves the private storage data at a specified contract address and storage slot. Returns only data
   * encrypted for the specified owner that has been already decrypted by the PXE Service. Note that there
   * may be multiple notes for a user in a single slot.
   *
   * @param owner - The address for whom the private data is encrypted.
   * @param contract - The AztecAddress of the target contract.
   * @param storageSlot - The storage slot to be fetched.
   * @returns A set of note preimages for the owner in that contract and slot.
   * @throws If the contract is not deployed.
   */
  getPrivateStorageAt(owner: AztecAddress, contract: AztecAddress, storageSlot: Fr): Promise<NotePreimage[]>;

  /**
   * Retrieves the public storage data at a specified contract address and storage slot.
   *
   * @param contract - The AztecAddress of the target contract.
   * @param storageSlot - The Fr representing the storage slot to be fetched.
   * @returns A buffer containing the public storage data at the storage slot.
   * @throws If the contract is not deployed.
   */
  getPublicStorageAt(contract: AztecAddress, storageSlot: Fr): Promise<Buffer | undefined>;

  /**
   * Adds a note to the database. Throw if the note hash of the note doesn't exist in the tree.
   * @param contract - The contract address of the note.
   * @param storageSlot - The storage slot of the note.
   * @param preimage - The note preimage.
   * @param nonce - The nonce of the note.
   * @param account - The public key of the account the note is associated with.
   */
  addNote(
    contract: AztecAddress,
    storageSlot: Fr,
    preimage: NotePreimage,
    nonce: Fr,
    account: PublicKey,
  ): Promise<void>;

  /**
   * Finds the nonce(s) for a note in a tx with given preimage at a specified contract address and storage slot.
   * @param contract - The contract address of the note.
   * @param storageSlot - The storage slot of the note.
   * @param preimage - The note preimage.
   * @param txHash - The tx hash of the tx containing the note.
   * @returns The nonces of the note.
   * @remarks More than single nonce may be returned since there might be more than one note with the same preimage.
   */
  getNoteNonces(contract: AztecAddress, storageSlot: Fr, preimage: NotePreimage, txHash: TxHash): Promise<Fr[]>;

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
   * Gets unencrypted public logs from the specified block range. Logs are grouped by block and then by
   * transaction. Use the `L2BlockL2Logs.unrollLogs` helper function to get an flattened array of logs instead.
   *
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
   * Returns the information about the server's node. Includes current Sandbox version, compatible Noir version,
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
}
// docs:end:pxe-interface
