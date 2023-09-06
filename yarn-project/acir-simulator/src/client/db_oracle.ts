import { CompleteAddress, HistoricBlockData, PrivateKey, PublicKey } from '@aztec/circuits.js';
import { FunctionAbi, FunctionDebugMetadata, FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

import { CommitmentsDB } from '../index.js';

/**
 * Information about a note needed during execution.
 */
export interface NoteData {
  /** The contract address of the note. */
  contractAddress: AztecAddress;
  /** The storage slot of the note. */
  storageSlot: Fr;
  /** The nonce of the note. */
  nonce: Fr;
  /** The preimage of the note */
  preimage: Fr[];
  /** The corresponding nullifier of the note */
  siloedNullifier?: Fr;
  /** The note's leaf index in the private data tree. Undefined for pending notes. */
  index?: bigint;
}

/**
 * Information about a note needed during execution.
 */
export interface PendingNoteData extends NoteData {
  /** The inner note hash (used as a nullified commitment). */
  innerNoteHash: Fr;
}

/**
 * The format that noir uses to get L1 to L2 Messages.
 */
export interface MessageLoadOracleInputs {
  /**
   * An collapsed array of fields containing all of the l1 to l2 message components.
   * `l1ToL2Message.toFieldArray()` -\> [sender, chainId, recipient, version, content, secretHash, deadline, fee]
   */
  message: Fr[];
  /**
   * The path in the merkle tree to the message.
   */
  siblingPath: Fr[];
  /**
   * The index of the message commitment in the merkle tree.
   */
  index: bigint;
}

/**
 * The format noir uses to get commitments.
 */
export interface CommitmentDataOracleInputs {
  /** The siloed commitment. */
  commitment: Fr;
  /**
   * The path in the merkle tree to the commitment.
   */
  siblingPath: Fr[];
  /**
   * The index of the message commitment in the merkle tree.
   */
  index: bigint;
}

/**
 * A function ABI with optional debug metadata
 */
export interface FunctionAbiWithDebugMetadata extends FunctionAbi {
  /**
   * Debug metadata for the function.
   */
  debug?: FunctionDebugMetadata;
}

/**
 * The database oracle interface.
 */
export interface DBOracle extends CommitmentsDB {
  /**
   * Retrieve the complete address associated to a given address.
   * @param address - Address to fetch the pubkey for.
   * @returns A complete address associated with the input address.
   */
  getCompleteAddress(address: AztecAddress): Promise<CompleteAddress>;

  /**
   * Retrieve the auth witness for a given message hash.
   * @param message_hash - The message hash.
   * @returns A Promise that resolves to an array of field elements representing the auth witness.
   */
  getAuthWitness(message_hash: Fr): Promise<Fr[]>;

  /**
   * Retrieve the secret key associated with a specific public key.
   * The function only allows access to the secret keys of the transaction creator,
   * and throws an error if the address does not match the public key address of the key pair.
   *
   * @param contractAddress - The contract address. Ignored here. But we might want to return different keys for different contracts.
   * @param pubKey - The public key of an account.
   * @returns A Promise that resolves to the secret key as a Buffer.
   * @throws An Error if the input address does not match the public key address of the key pair.
   */
  getSecretKey(contractAddress: AztecAddress, pubKey: PublicKey): Promise<PrivateKey>;

  /**
   * Retrieves a set of notes stored in the database for a given contract address and storage slot.
   * The query result is paginated using 'limit' and 'offset' values.
   * Returns an object containing an array of note data, including preimage, nonce, and index for each note.
   *
   * @param contractAddress - The AztecAddress instance representing the contract address.
   * @param storageSlot - The Fr instance representing the storage slot of the notes.
   * @returns A Promise that resolves to an array of note data.
   */
  getNotes(contractAddress: AztecAddress, storageSlot: Fr): Promise<NoteData[]>;

  /**
   * Retrieve the ABI information of a specific function within a contract.
   * The function is identified by its selector, which is a unique identifier generated from the function signature.
   *
   * @param contractAddress - The contract address.
   * @param selector - The corresponding function selector.
   * @returns A Promise that resolves to a FunctionAbi object containing the ABI information of the target function.
   */
  getFunctionABI(contractAddress: AztecAddress, selector: FunctionSelector): Promise<FunctionAbiWithDebugMetadata>;

  /**
   * Retrieves the portal contract address associated with the given contract address.
   * Throws an error if the input contract address is not found or invalid.
   *
   * @param contractAddress - The address of the contract whose portal address is to be fetched.
   * @returns A Promise that resolves to an EthAddress instance, representing the portal contract address.
   */
  getPortalContractAddress(contractAddress: AztecAddress): Promise<EthAddress>;

  /**
   * Retrieve the databases view of the Historic Block Data object.
   * This structure is fed into the circuits simulator and is used to prove against certain historic roots.
   *
   * @returns A Promise that resolves to a HistoricBlockData object.
   */
  getHistoricBlockData(): Promise<HistoricBlockData>;
}
