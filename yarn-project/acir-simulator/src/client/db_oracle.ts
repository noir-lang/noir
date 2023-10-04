import { CompleteAddress, GrumpkinPrivateKey, HistoricBlockData, PublicKey } from '@aztec/circuits.js';
import { FunctionAbi, FunctionDebugMetadata, FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

import { NoteData } from '../acvm/index.js';
import { CommitmentsDB } from '../public/index.js';

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
   * @param messageHash - The message hash.
   * @returns A Promise that resolves to an array of field elements representing the auth witness.
   */
  getAuthWitness(messageHash: Fr): Promise<Fr[]>;

  /**
   * Retrieve the secret key associated with a specific public key.
   * The function only allows access to the secret keys of the transaction creator,
   * and throws an error if the address does not match the public key address of the key pair.
   *
   * @param contractAddress - The contract address. Ignored here. But we might want to return different keys for different contracts.
   * @param pubKey - The public key of an account.
   * @returns A Promise that resolves to the secret key.
   * @throws An Error if the input address does not match the public key address of the key pair.
   */
  getSecretKey(contractAddress: AztecAddress, pubKey: PublicKey): Promise<GrumpkinPrivateKey>;

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
   * Gets the index of a nullifier in the nullifier tree.
   * @param nullifier - The nullifier.
   * @returns - The index of the nullifier. Undefined if it does not exist in the tree.
   */
  getNullifierIndex(nullifier: Fr): Promise<bigint | undefined>;

  /**
   * Retrieve the databases view of the Historic Block Data object.
   * This structure is fed into the circuits simulator and is used to prove against certain historic roots.
   *
   * @returns A Promise that resolves to a HistoricBlockData object.
   */
  getHistoricBlockData(): Promise<HistoricBlockData>;
}
