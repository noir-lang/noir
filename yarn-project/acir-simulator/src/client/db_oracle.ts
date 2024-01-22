import { L2Block, MerkleTreeId, NullifierMembershipWitness, PublicDataWitness } from '@aztec/circuit-types';
import { BlockHeader, CompleteAddress } from '@aztec/circuits.js';
import { FunctionArtifactWithDebugMetadata, FunctionSelector } from '@aztec/foundation/abi';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { Fr } from '@aztec/foundation/fields';

import { KeyPair, NoteData } from '../acvm/index.js';
import { CommitmentsDB } from '../public/db.js';

/**
 * Error thrown when a contract is not found in the database.
 */
export class ContractNotFoundError extends Error {
  constructor(contractAddress: string) {
    super(`DB has no contract with address ${contractAddress}`);
  }
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
   * Retrieve a capsule from the capsule dispenser.
   * @returns A promise that resolves to an array of field elements representing the capsule.
   * @remarks A capsule is a "blob" of data that is passed to the contract through an oracle.
   */
  popCapsule(): Promise<Fr[]>;

  /**
   * Retrieve the nullifier key pair associated with a specific account.
   * The function only allows access to the secret keys of the transaction creator,
   * and throws an error if the address does not match the account address of the key pair.
   *
   * @param accountAddress - The account address.
   * @param contractAddress - The contract address.
   * @returns A Promise that resolves to the nullifier key pair.
   * @throws An Error if the input address does not match the account address of the key pair.
   */
  getNullifierKeyPair(accountAddress: AztecAddress, contractAddress: AztecAddress): Promise<KeyPair>;

  /**
   * Retrieves a set of notes stored in the database for a given contract address and storage slot.
   * The query result is paginated using 'limit' and 'offset' values.
   * Returns an object containing an array of note data.
   *
   * @param contractAddress - The AztecAddress instance representing the contract address.
   * @param storageSlot - The Fr instance representing the storage slot of the notes.
   * @returns A Promise that resolves to an array of note data.
   */
  getNotes(contractAddress: AztecAddress, storageSlot: Fr): Promise<NoteData[]>;

  /**
   * Retrieve the artifact information of a specific function within a contract.
   * The function is identified by its selector, which is a unique identifier generated from the function signature.
   *
   * @param contractAddress - The contract address.
   * @param selector - The corresponding function selector.
   * @returns A Promise that resolves to a FunctionArtifact object.
   */
  getFunctionArtifact(
    contractAddress: AztecAddress,
    selector: FunctionSelector,
  ): Promise<FunctionArtifactWithDebugMetadata>;

  /**
   * Retrieves the artifact of a specified function within a given contract.
   * The function is identified by its name, which is unique within a contract.
   *
   * @param contractAddress - The AztecAddress representing the contract containing the function.
   * @param functionName - The name of the function.
   * @returns The corresponding function's artifact as an object.
   */
  getFunctionArtifactByName(
    contractAddress: AztecAddress,
    functionName: string,
  ): Promise<FunctionArtifactWithDebugMetadata | undefined>;

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
   * Retrieve the databases view of the Block Header object.
   * This structure is fed into the circuits simulator and is used to prove against certain historical roots.
   *
   * @returns A Promise that resolves to a BlockHeader object.
   */
  getBlockHeader(): Promise<BlockHeader>;

  /**
   * Fetch the index of the leaf in the respective tree
   * @param blockNumber - The block number at which to get the leaf index.
   * @param treeId - The id of the tree to search.
   * @param leafValue - The leaf value buffer.
   * @returns - The index of the leaf. Undefined if it does not exist in the tree.
   */
  findLeafIndex(blockNumber: number, treeId: MerkleTreeId, leafValue: Fr): Promise<bigint | undefined>;

  /**
   * Fetch the sibling path of the leaf in the respective tree
   * @param blockNumber - The block number at which to get the sibling path.
   * @param treeId - The id of the tree to search.
   * @param leafIndex - The index of the leaf.
   * @returns - The sibling path of the leaf.
   */
  getSiblingPath(blockNumber: number, treeId: MerkleTreeId, leafIndex: bigint): Promise<Fr[]>;

  /**
   * Returns a nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the index.
   * @param nullifier - Nullifier we try to find witness for.
   * @returns The nullifier membership witness (if found).
   */
  getNullifierMembershipWitness(blockNumber: number, nullifier: Fr): Promise<NullifierMembershipWitness | undefined>;

  /**
   * Returns a low nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the index.
   * @param nullifier - Nullifier we try to find the low nullifier witness for.
   * @returns The low nullifier membership witness (if found).
   * @remarks Low nullifier witness can be used to perform a nullifier non-inclusion proof by leveraging the "linked
   * list structure" of leaves and proving that a lower nullifier is pointing to a bigger next value than the nullifier
   * we are trying to prove non-inclusion for.
   */
  getLowNullifierMembershipWitness(blockNumber: number, nullifier: Fr): Promise<NullifierMembershipWitness | undefined>;

  /**
   * Returns a witness for a given slot of the public data tree at a given block.
   * @param blockNumber - The block number at which to get the witness.
   * @param leafSlot - The slot of the public data in the public data tree.
   */
  getPublicDataTreeWitness(blockNumber: number, leafSlot: Fr): Promise<PublicDataWitness | undefined>;

  /**
   * Fetch a block corresponding to the given block number.
   * @param blockNumber - The block number of a block to fetch.
   * @returns - The block corresponding to the given block number. Undefined if it does not exist.
   */
  getBlock(blockNumber: number): Promise<L2Block | undefined>;

  /**
   * Fetches the current block number.
   * @returns The block number.
   */
  getBlockNumber(): Promise<number>;
}
