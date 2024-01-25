import {
  AuthWitness,
  AztecNode,
  CompleteAddress,
  INITIAL_L2_BLOCK_NUM,
  MerkleTreeId,
  NullifierMembershipWitness,
  PublicDataWitness,
} from '@aztec/circuit-types';
import { BlockHeader } from '@aztec/circuits.js';
import { computeGlobalsHash, siloNullifier } from '@aztec/circuits.js/abis';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';

import { NoteData, TypedOracle } from '../acvm/index.js';
import { DBOracle } from './db_oracle.js';
import { pickNotes } from './pick_notes.js';

/**
 * The execution context for a client view tx simulation.
 * It only reads data from data sources. Nothing will be updated or created during this simulation.
 */
export class ViewDataOracle extends TypedOracle {
  constructor(
    protected readonly contractAddress: AztecAddress,
    /** Data required to reconstruct the block hash, it contains historical roots. */
    protected readonly blockHeader: BlockHeader,
    /** List of transient auth witnesses to be used during this simulation */
    protected readonly authWitnesses: AuthWitness[],
    protected readonly db: DBOracle,
    protected readonly aztecNode: AztecNode | undefined,
    protected log = createDebugLogger('aztec:simulator:client_view_context'),
  ) {
    super();
  }

  /**
   * Return the nullifier key pair of an account to use in a specific contract.
   * @param account - The account address of the nullifier key.
   */
  public getNullifierKeyPair(account: AztecAddress) {
    return this.db.getNullifierKeyPair(account, this.contractAddress);
  }

  /**
   * Fetches the index and sibling path of a leaf at a given block from a given tree.
   * @param blockNumber - The block number at which to get the membership witness.
   * @param treeId - Id of the tree to get the sibling path from.
   * @param leafValue - The leaf value
   * @returns The index and sibling path concatenated [index, sibling_path]
   */
  public async getMembershipWitness(blockNumber: number, treeId: MerkleTreeId, leafValue: Fr): Promise<Fr[]> {
    const index = await this.db.findLeafIndex(blockNumber, treeId, leafValue);
    if (!index) {
      throw new Error(`Leaf value: ${leafValue} not found in ${MerkleTreeId[treeId]}`);
    }
    const siblingPath = await this.db.getSiblingPath(blockNumber, treeId, index);
    return [new Fr(index), ...siblingPath];
  }

  /**
   * Fetches a sibling path at a given block and index from a tree specified by `treeId`.
   * @param blockNumber - The block number at which to get the membership witness.
   * @param treeId - Id of the tree to get the sibling path from.
   * @param leafIndex - Index of the leaf to get sibling path for
   * @returns The sibling path.
   */
  public getSiblingPath(blockNumber: number, treeId: MerkleTreeId, leafIndex: Fr): Promise<Fr[]> {
    return this.db.getSiblingPath(blockNumber, treeId, leafIndex.toBigInt());
  }

  /**
   * Returns a nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the index.
   * @param nullifier - Nullifier we try to find witness for.
   * @returns The nullifier membership witness (if found).
   */
  public async getNullifierMembershipWitness(
    blockNumber: number,
    nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    return await this.db.getNullifierMembershipWitness(blockNumber, nullifier);
  }

  /**
   * Returns a low nullifier membership witness for a given nullifier at a given block.
   * @param blockNumber - The block number at which to get the index.
   * @param nullifier - Nullifier we try to find the low nullifier witness for.
   * @returns The low nullifier membership witness (if found).
   * @remarks Low nullifier witness can be used to perform a nullifier non-inclusion proof by leveraging the "linked
   * list structure" of leaves and proving that a lower nullifier is pointing to a bigger next value than the nullifier
   * we are trying to prove non-inclusion for.
   */
  public async getLowNullifierMembershipWitness(
    blockNumber: number,
    nullifier: Fr,
  ): Promise<NullifierMembershipWitness | undefined> {
    return await this.db.getLowNullifierMembershipWitness(blockNumber, nullifier);
  }

  /**
   * Returns a public data tree witness for a given leaf slot at a given block.
   * @param blockNumber - The block number at which to get the index.
   * @param leafSlot - The slot of the public data tree to get the witness for.
   * @returns - The witness
   */
  public async getPublicDataTreeWitness(blockNumber: number, leafSlot: Fr): Promise<PublicDataWitness | undefined> {
    return await this.db.getPublicDataTreeWitness(blockNumber, leafSlot);
  }

  /**
   * Fetches a block header of a given block.
   * @param blockNumber - The number of a block of which to get the block header.
   * @returns Block extracted from a block with block number `blockNumber`.
   */
  public async getBlockHeader(blockNumber: number): Promise<BlockHeader | undefined> {
    const block = await this.db.getBlock(blockNumber);
    if (!block) {
      return undefined;
    }
    return new BlockHeader(
      block.header.state.partial.noteHashTree.root,
      block.header.state.partial.nullifierTree.root,
      block.header.state.partial.contractTree.root,
      block.header.state.l1ToL2MessageTree.root,
      block.archive.root,
      new Fr(0), // TODO(#3441) privateKernelVkTreeRoot is not present in L2Block and it's not yet populated in noir
      block.header.state.partial.publicDataTree.root,
      computeGlobalsHash(block.header.globalVariables),
    );
  }

  /**
   * Gets number of a block in which a given nullifier tree root was included.
   * @param nullifierTreeRoot - The nullifier tree root to get the block number for.
   * @returns The block number.
   *
   * TODO(#3564) - Nuke this oracle and inject the number directly to context
   */
  public async getNullifierRootBlockNumber(nullifierTreeRoot: Fr): Promise<number | undefined> {
    const currentBlockNumber = await this.db.getBlockNumber();
    for (let i = currentBlockNumber; i >= INITIAL_L2_BLOCK_NUM; i -= 2) {
      const block = await this.db.getBlock(i);
      if (!block) {
        throw new Error(`Block ${i} not found`);
      }
      if (block.header.state.partial.nullifierTree.root.equals(nullifierTreeRoot)) {
        return i;
      }
      if (block.header.state.partial.nullifierTree.root.equals(nullifierTreeRoot)) {
        return i - 1;
      }
    }
    throw new Error(`Failed to find block containing nullifier tree root ${nullifierTreeRoot}`);
  }

  /**
   * Retrieve the complete address associated to a given address.
   * @param address - Address to fetch the complete address for.
   * @returns A complete address associated with the input address.
   */
  public getCompleteAddress(address: AztecAddress): Promise<CompleteAddress> {
    return this.db.getCompleteAddress(address);
  }

  /**
   * Returns an auth witness for the given message hash. Checks on the list of transient witnesses
   * for this transaction first, and falls back to the local database if not found.
   * @param messageHash - Hash of the message to authenticate.
   * @returns Authentication witness for the requested message hash.
   */
  public getAuthWitness(messageHash: Fr): Promise<Fr[] | undefined> {
    return Promise.resolve(
      this.authWitnesses.find(w => w.requestHash.equals(messageHash))?.witness ?? this.db.getAuthWitness(messageHash),
    );
  }

  /**
   * Pops a capsule from the capsule dispenser
   * @returns The capsule values
   * @remarks A capsule is a "blob" of data that is passed to the contract through an oracle.
   */
  public popCapsule(): Promise<Fr[]> {
    return this.db.popCapsule();
  }

  /**
   * Gets some notes for a contract address and storage slot.
   * Returns a flattened array containing filtered notes.
   *
   * @remarks
   * Check for pending notes with matching slot.
   * Real notes coming from DB will have a leafIndex which
   * represents their index in the note hash tree.
   *
   * @param storageSlot - The storage slot.
   * @param numSelects - The number of valid selects in selectBy and selectValues.
   * @param selectBy - An array of indices of the fields to selects.
   * @param selectValues - The values to match.
   * @param selectComparators - The comparators to use to match values.
   * @param sortBy - An array of indices of the fields to sort.
   * @param sortOrder - The order of the corresponding index in sortBy. (1: DESC, 2: ASC, 0: Do nothing)
   * @param limit - The number of notes to retrieve per query.
   * @param offset - The starting index for pagination.
   * @returns Array of note data.
   */
  public async getNotes(
    storageSlot: Fr,
    numSelects: number,
    selectBy: number[],
    selectValues: Fr[],
    selectComparators: number[],
    sortBy: number[],
    sortOrder: number[],
    limit: number,
    offset: number,
  ): Promise<NoteData[]> {
    const dbNotes = await this.db.getNotes(this.contractAddress, storageSlot);
    return pickNotes<NoteData>(dbNotes, {
      selects: selectBy
        .slice(0, numSelects)
        .map((index, i) => ({ index, value: selectValues[i], comparator: selectComparators[i] })),
      sorts: sortBy.map((index, i) => ({ index, order: sortOrder[i] })),
      limit,
      offset,
    });
  }

  /**
   * Check if a nullifier exists in the nullifier tree.
   * @param innerNullifier - The inner nullifier.
   * @returns A boolean indicating whether the nullifier exists in the tree or not.
   */
  public async checkNullifierExists(innerNullifier: Fr) {
    const nullifier = siloNullifier(this.contractAddress, innerNullifier!);
    const index = await this.db.getNullifierIndex(nullifier);
    return index !== undefined;
  }

  /**
   * Fetches the a message from the db, given its key.
   * @param msgKey - A buffer representing the message key.
   * @returns The l1 to l2 message data
   */
  public async getL1ToL2Message(msgKey: Fr) {
    return await this.db.getL1ToL2Message(msgKey);
  }

  /**
   * Retrieves the portal contract address associated with the given contract address.
   * Throws an error if the input contract address is not found or invalid.
   * @param contractAddress - The address of the contract whose portal address is to be fetched.
   * @returns The portal contract address.
   */
  public getPortalContractAddress(contractAddress: AztecAddress) {
    return this.db.getPortalContractAddress(contractAddress);
  }

  /**
   * Read the public storage data.
   * @param startStorageSlot - The starting storage slot.
   * @param numberOfElements - Number of elements to read from the starting storage slot.
   */
  public async storageRead(startStorageSlot: Fr, numberOfElements: number) {
    if (!this.aztecNode) {
      throw new Error('Aztec node is undefined, cannot read storage.');
    }

    const values = [];
    for (let i = 0n; i < numberOfElements; i++) {
      const storageSlot = new Fr(startStorageSlot.value + i);
      const value = await this.aztecNode.getPublicStorageAt(this.contractAddress, storageSlot);

      this.log(`Oracle storage read: slot=${storageSlot.toString()} value=${value}`);
      values.push(value);
    }
    return values;
  }
}
