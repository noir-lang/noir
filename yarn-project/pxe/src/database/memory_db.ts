import { BlockHeader, CompleteAddress, PublicKey } from '@aztec/circuits.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { createDebugLogger } from '@aztec/foundation/log';
import { MerkleTreeId, NoteFilter } from '@aztec/types';

import { MemoryContractDatabase } from '../contract_database/index.js';
import { Database } from './database.js';
import { NoteDao } from './note_dao.js';

/**
 * The MemoryDB class provides an in-memory implementation of a database to manage transactions and auxiliary data.
 * It extends the MemoryContractDatabase, allowing it to store contract-related data as well.
 * The class offers methods to add, fetch, and remove transaction records and auxiliary data based on various filters such as transaction hash, address, and storage slot.
 * As an in-memory database, the stored data will not persist beyond the life of the application instance.
 */
export class MemoryDB extends MemoryContractDatabase implements Database {
  private notesTable: NoteDao[] = [];
  private treeRoots: Record<MerkleTreeId, Fr> | undefined;
  private globalVariablesHash: Fr | undefined;
  private addresses: CompleteAddress[] = [];
  private authWitnesses: Record<string, Fr[]> = {};
  // A capsule is a "blob" of data that is passed to the contract through an oracle.
  // We are using a stack to keep track of the capsules that are passed to the contract.
  private capsuleStack: Fr[][] = [];

  constructor(logSuffix?: string) {
    super(createDebugLogger(logSuffix ? 'aztec:memory_db_' + logSuffix : 'aztec:memory_db'));
  }

  /**
   * Add a auth witness to the database.
   * @param messageHash - The message hash.
   * @param witness - An array of field elements representing the auth witness.
   */
  public addAuthWitness(messageHash: Fr, witness: Fr[]): Promise<void> {
    this.authWitnesses[messageHash.toString()] = witness;
    return Promise.resolve();
  }

  /**
   * Fetching the auth witness for a given message hash.
   * @param messageHash - The message hash.
   * @returns A Promise that resolves to an array of field elements representing the auth witness.
   */
  public getAuthWitness(messageHash: Fr): Promise<Fr[]> {
    return Promise.resolve(this.authWitnesses[messageHash.toString()]);
  }

  public addNote(note: NoteDao): Promise<void> {
    this.notesTable.push(note);
    return Promise.resolve();
  }

  public addCapsule(capsule: Fr[]): Promise<void> {
    this.capsuleStack.push(capsule);
    return Promise.resolve();
  }

  public popCapsule(): Promise<Fr[] | undefined> {
    return Promise.resolve(this.capsuleStack.pop());
  }

  public addNotes(notes: NoteDao[]) {
    this.notesTable.push(...notes);
    return Promise.resolve();
  }

  public async getNotes(filter: NoteFilter): Promise<NoteDao[]> {
    let ownerPublicKey: PublicKey | undefined;
    if (filter.owner !== undefined) {
      const ownerCompleteAddress = await this.getCompleteAddress(filter.owner);
      if (ownerCompleteAddress === undefined) {
        throw new Error(`Owner ${filter.owner.toString()} not found in memory database`);
      }
      ownerPublicKey = ownerCompleteAddress.publicKey;
    }

    return this.notesTable.filter(
      note =>
        (filter.contractAddress == undefined || note.contractAddress.equals(filter.contractAddress)) &&
        (filter.txHash == undefined || note.txHash.equals(filter.txHash)) &&
        (filter.storageSlot == undefined || note.storageSlot.equals(filter.storageSlot!)) &&
        (ownerPublicKey == undefined || note.publicKey.equals(ownerPublicKey!)),
    );
  }

  public removeNullifiedNotes(nullifiers: Fr[], account: PublicKey) {
    const nullifierSet = new Set(nullifiers.map(nullifier => nullifier.toString()));
    const [remaining, removed] = this.notesTable.reduce(
      (acc: [NoteDao[], NoteDao[]], note) => {
        const nullifier = note.siloedNullifier.toString();
        if (note.publicKey.equals(account) && nullifierSet.has(nullifier)) {
          acc[1].push(note);
        } else {
          acc[0].push(note);
        }
        return acc;
      },
      [[], []],
    );

    this.notesTable = remaining;

    return Promise.resolve(removed);
  }

  public getTreeRoots(): Record<MerkleTreeId, Fr> {
    const roots = this.treeRoots;
    if (!roots) {
      throw new Error(`Tree roots not set in memory database`);
    }
    return roots;
  }

  public setTreeRoots(roots: Record<MerkleTreeId, Fr>) {
    this.treeRoots = roots;
    return Promise.resolve();
  }

  public getBlockHeader(): BlockHeader {
    const roots = this.getTreeRoots();
    if (!this.globalVariablesHash) {
      throw new Error(`Global variables hash not set in memory database`);
    }
    return new BlockHeader(
      roots[MerkleTreeId.NOTE_HASH_TREE],
      roots[MerkleTreeId.NULLIFIER_TREE],
      roots[MerkleTreeId.CONTRACT_TREE],
      roots[MerkleTreeId.L1_TO_L2_MESSAGES_TREE],
      roots[MerkleTreeId.ARCHIVE],
      Fr.ZERO, // todo: private kernel vk tree root
      roots[MerkleTreeId.PUBLIC_DATA_TREE],
      this.globalVariablesHash,
    );
  }

  public async setBlockHeader(blockHeader: BlockHeader): Promise<void> {
    this.globalVariablesHash = blockHeader.globalVariablesHash;
    await this.setTreeRoots({
      [MerkleTreeId.NOTE_HASH_TREE]: blockHeader.noteHashTreeRoot,
      [MerkleTreeId.NULLIFIER_TREE]: blockHeader.nullifierTreeRoot,
      [MerkleTreeId.CONTRACT_TREE]: blockHeader.contractTreeRoot,
      [MerkleTreeId.L1_TO_L2_MESSAGES_TREE]: blockHeader.l1ToL2MessagesTreeRoot,
      [MerkleTreeId.ARCHIVE]: blockHeader.archiveRoot,
      [MerkleTreeId.PUBLIC_DATA_TREE]: blockHeader.publicDataTreeRoot,
    });
  }

  public addCompleteAddress(completeAddress: CompleteAddress): Promise<boolean> {
    const accountIndex = this.addresses.findIndex(r => r.address.equals(completeAddress.address));
    if (accountIndex !== -1) {
      if (this.addresses[accountIndex].equals(completeAddress)) {
        return Promise.resolve(false);
      }

      throw new Error(
        `Complete address with aztec address ${completeAddress.address.toString()} but different public key or partial key already exists in memory database`,
      );
    }
    this.addresses.push(completeAddress);
    return Promise.resolve(true);
  }

  public getCompleteAddress(address: AztecAddress): Promise<CompleteAddress | undefined> {
    const recipient = this.addresses.find(r => r.address.equals(address));
    return Promise.resolve(recipient);
  }

  public getCompleteAddresses(): Promise<CompleteAddress[]> {
    return Promise.resolve(this.addresses);
  }

  public estimateSize() {
    const notesSize = this.notesTable.reduce((sum, note) => sum + note.getSize(), 0);
    const treeRootsSize = this.treeRoots ? Object.entries(this.treeRoots).length * Fr.SIZE_IN_BYTES : 0;
    const authWits = Object.entries(this.authWitnesses);
    const authWitsSize = authWits.reduce((sum, [key, value]) => sum + key.length + value.length * Fr.SIZE_IN_BYTES, 0);
    return notesSize + treeRootsSize + authWitsSize + this.addresses.length * CompleteAddress.SIZE_IN_BYTES;
  }
}
