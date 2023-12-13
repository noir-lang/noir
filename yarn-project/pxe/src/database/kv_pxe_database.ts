import { AztecAddress, BlockHeader, CompleteAddress } from '@aztec/circuits.js';
import { Fr } from '@aztec/foundation/fields';
import { AztecArray, AztecKVStore, AztecMap, AztecMultiMap, AztecSingleton } from '@aztec/kv-store';
import { ContractDao, MerkleTreeId, NoteFilter, PublicKey } from '@aztec/types';

import { NoteDao } from './note_dao.js';
import { PxeDatabase } from './pxe_database.js';

/** Serialized structure of a block header */
type SerializedBlockHeader = {
  /** The tree roots when the block was created */
  roots: Record<MerkleTreeId, string>;
  /** The hash of the global variables */
  globalVariablesHash: string;
};

/**
 * A PXE database backed by LMDB.
 */
export class KVPxeDatabase implements PxeDatabase {
  #blockHeader: AztecSingleton<SerializedBlockHeader>;
  #addresses: AztecArray<Buffer>;
  #addressIndex: AztecMap<string, number>;
  #authWitnesses: AztecMap<string, Buffer[]>;
  #capsules: AztecArray<Buffer[]>;
  #contracts: AztecMap<string, Buffer>;
  #notes: AztecArray<Buffer>;
  #nullifiedNotes: AztecMap<number, boolean>;
  #notesByContract: AztecMultiMap<string, number>;
  #notesByStorageSlot: AztecMultiMap<string, number>;
  #notesByTxHash: AztecMultiMap<string, number>;
  #notesByOwner: AztecMultiMap<string, number>;
  #db: AztecKVStore;

  constructor(db: AztecKVStore) {
    this.#db = db;

    this.#addresses = db.createArray('addresses');
    this.#addressIndex = db.createMap('address_index');

    this.#authWitnesses = db.createMap('auth_witnesses');
    this.#capsules = db.createArray('capsules');
    this.#blockHeader = db.createSingleton('block_header');
    this.#contracts = db.createMap('contracts');

    this.#notes = db.createArray('notes');
    this.#nullifiedNotes = db.createMap('nullified_notes');

    this.#notesByContract = db.createMultiMap('notes_by_contract');
    this.#notesByStorageSlot = db.createMultiMap('notes_by_storage_slot');
    this.#notesByTxHash = db.createMultiMap('notes_by_tx_hash');
    this.#notesByOwner = db.createMultiMap('notes_by_owner');
  }

  async addAuthWitness(messageHash: Fr, witness: Fr[]): Promise<void> {
    await this.#authWitnesses.set(
      messageHash.toString(),
      witness.map(w => w.toBuffer()),
    );
  }

  getAuthWitness(messageHash: Fr): Promise<Fr[] | undefined> {
    const witness = this.#authWitnesses.get(messageHash.toString());
    return Promise.resolve(witness?.map(w => Fr.fromBuffer(w)));
  }

  async addCapsule(capsule: Fr[]): Promise<void> {
    await this.#capsules.push(capsule.map(c => c.toBuffer()));
  }

  async popCapsule(): Promise<Fr[] | undefined> {
    const val = await this.#capsules.pop();
    return val?.map(b => Fr.fromBuffer(b));
  }

  async addNote(note: NoteDao): Promise<void> {
    await this.addNotes([note]);
  }

  async addNotes(notes: NoteDao[]): Promise<void> {
    const newLength = await this.#notes.push(...notes.map(note => note.toBuffer()));
    for (const [index, note] of notes.entries()) {
      const noteId = newLength - notes.length + index;
      await Promise.all([
        this.#notesByContract.set(note.contractAddress.toString(), noteId),
        this.#notesByStorageSlot.set(note.storageSlot.toString(), noteId),
        this.#notesByTxHash.set(note.txHash.toString(), noteId),
        this.#notesByOwner.set(note.publicKey.toString(), noteId),
      ]);
    }
  }

  *#getAllNonNullifiedNotes(): IterableIterator<NoteDao> {
    for (const [index, serialized] of this.#notes.entries()) {
      if (this.#nullifiedNotes.has(index)) {
        continue;
      }

      yield NoteDao.fromBuffer(serialized);
    }
  }

  async getNotes(filter: NoteFilter): Promise<NoteDao[]> {
    const publicKey: PublicKey | undefined = filter.owner
      ? (await this.getCompleteAddress(filter.owner))?.publicKey
      : undefined;

    const initialNoteIds = publicKey
      ? this.#notesByOwner.getValues(publicKey.toString())
      : filter.txHash
      ? this.#notesByTxHash.getValues(filter.txHash.toString())
      : filter.contractAddress
      ? this.#notesByContract.getValues(filter.contractAddress.toString())
      : filter.storageSlot
      ? this.#notesByStorageSlot.getValues(filter.storageSlot.toString())
      : undefined;

    if (!initialNoteIds) {
      return Array.from(this.#getAllNonNullifiedNotes());
    }

    const result: NoteDao[] = [];
    for (const noteId of initialNoteIds) {
      const serializedNote = this.#notes.at(noteId);
      if (!serializedNote) {
        continue;
      }

      const note = NoteDao.fromBuffer(serializedNote);
      if (filter.contractAddress && !note.contractAddress.equals(filter.contractAddress)) {
        continue;
      }

      if (filter.txHash && !note.txHash.equals(filter.txHash)) {
        continue;
      }

      if (filter.storageSlot && !note.storageSlot.equals(filter.storageSlot!)) {
        continue;
      }

      if (publicKey && !note.publicKey.equals(publicKey)) {
        continue;
      }

      result.push(note);
    }

    return result;
  }

  removeNullifiedNotes(nullifiers: Fr[], account: PublicKey): Promise<NoteDao[]> {
    const nullifierSet = new Set(nullifiers.map(n => n.toString()));
    return this.#db.transaction(() => {
      const notesIds = this.#notesByOwner.getValues(account.toString());
      const nullifiedNotes: NoteDao[] = [];

      for (const noteId of notesIds) {
        const note = NoteDao.fromBuffer(this.#notes.at(noteId)!);
        if (nullifierSet.has(note.siloedNullifier.toString())) {
          nullifiedNotes.push(note);

          void this.#nullifiedNotes.set(noteId, true);
          void this.#notesByOwner.deleteValue(account.toString(), noteId);
          void this.#notesByTxHash.deleteValue(note.txHash.toString(), noteId);
          void this.#notesByContract.deleteValue(note.contractAddress.toString(), noteId);
          void this.#notesByStorageSlot.deleteValue(note.storageSlot.toString(), noteId);
        }
      }

      return nullifiedNotes;
    });
  }

  getTreeRoots(): Record<MerkleTreeId, Fr> {
    const roots = this.#blockHeader.get()?.roots;
    if (!roots) {
      throw new Error(`Tree roots not set`);
    }

    return {
      [MerkleTreeId.ARCHIVE]: Fr.fromString(roots[MerkleTreeId.ARCHIVE]),
      [MerkleTreeId.CONTRACT_TREE]: Fr.fromString(roots[MerkleTreeId.CONTRACT_TREE].toString()),
      [MerkleTreeId.L1_TO_L2_MESSAGES_TREE]: Fr.fromString(roots[MerkleTreeId.L1_TO_L2_MESSAGES_TREE].toString()),
      [MerkleTreeId.NOTE_HASH_TREE]: Fr.fromString(roots[MerkleTreeId.NOTE_HASH_TREE].toString()),
      [MerkleTreeId.PUBLIC_DATA_TREE]: Fr.fromString(roots[MerkleTreeId.PUBLIC_DATA_TREE].toString()),
      [MerkleTreeId.NULLIFIER_TREE]: Fr.fromString(roots[MerkleTreeId.NULLIFIER_TREE].toString()),
    };
  }

  async setBlockHeader(blockHeader: BlockHeader): Promise<void> {
    await this.#blockHeader.set({
      globalVariablesHash: blockHeader.globalVariablesHash.toString(),
      roots: {
        [MerkleTreeId.NOTE_HASH_TREE]: blockHeader.noteHashTreeRoot.toString(),
        [MerkleTreeId.NULLIFIER_TREE]: blockHeader.nullifierTreeRoot.toString(),
        [MerkleTreeId.CONTRACT_TREE]: blockHeader.contractTreeRoot.toString(),
        [MerkleTreeId.L1_TO_L2_MESSAGES_TREE]: blockHeader.l1ToL2MessagesTreeRoot.toString(),
        [MerkleTreeId.ARCHIVE]: blockHeader.archiveRoot.toString(),
        [MerkleTreeId.PUBLIC_DATA_TREE]: blockHeader.publicDataTreeRoot.toString(),
      },
    });
  }

  getBlockHeader(): BlockHeader {
    const value = this.#blockHeader.get();
    if (!value) {
      throw new Error(`Block header not set`);
    }

    const blockHeader = new BlockHeader(
      Fr.fromString(value.roots[MerkleTreeId.NOTE_HASH_TREE]),
      Fr.fromString(value.roots[MerkleTreeId.NULLIFIER_TREE]),
      Fr.fromString(value.roots[MerkleTreeId.CONTRACT_TREE]),
      Fr.fromString(value.roots[MerkleTreeId.L1_TO_L2_MESSAGES_TREE]),
      Fr.fromString(value.roots[MerkleTreeId.ARCHIVE]),
      Fr.ZERO, // todo: private kernel vk tree root
      Fr.fromString(value.roots[MerkleTreeId.PUBLIC_DATA_TREE]),
      Fr.fromString(value.globalVariablesHash),
    );

    return blockHeader;
  }

  addCompleteAddress(completeAddress: CompleteAddress): Promise<boolean> {
    return this.#db.transaction(() => {
      const addressString = completeAddress.address.toString();
      const buffer = completeAddress.toBuffer();
      const existing = this.#addressIndex.get(addressString);
      if (typeof existing === 'undefined') {
        const index = this.#addresses.length;
        void this.#addresses.push(buffer);
        void this.#addressIndex.set(addressString, index);

        return true;
      } else {
        const existingBuffer = this.#addresses.at(existing);

        if (existingBuffer?.equals(buffer)) {
          return false;
        }

        throw new Error(
          `Complete address with aztec address ${addressString} but different public key or partial key already exists in memory database`,
        );
      }
    });
  }

  getCompleteAddress(address: AztecAddress): Promise<CompleteAddress | undefined> {
    const index = this.#addressIndex.get(address.toString());
    if (typeof index === 'undefined') {
      return Promise.resolve(undefined);
    }

    const value = this.#addresses.at(index);
    return Promise.resolve(value ? CompleteAddress.fromBuffer(value) : undefined);
  }

  getCompleteAddresses(): Promise<CompleteAddress[]> {
    return Promise.resolve(Array.from(this.#addresses).map(v => CompleteAddress.fromBuffer(v)));
  }

  estimateSize(): number {
    const notesSize = Array.from(this.#getAllNonNullifiedNotes()).reduce((sum, note) => sum + note.getSize(), 0);
    const authWitsSize = Array.from(this.#authWitnesses.values()).reduce(
      (sum, value) => sum + value.length * Fr.SIZE_IN_BYTES,
      0,
    );
    const addressesSize = this.#addresses.length * CompleteAddress.SIZE_IN_BYTES;
    const treeRootsSize = Object.keys(MerkleTreeId).length * Fr.SIZE_IN_BYTES;

    return notesSize + treeRootsSize + authWitsSize + addressesSize;
  }

  async addContract(contract: ContractDao): Promise<void> {
    await this.#contracts.set(contract.completeAddress.address.toString(), contract.toBuffer());
  }

  getContract(address: AztecAddress): Promise<ContractDao | undefined> {
    const contract = this.#contracts.get(address.toString());
    return Promise.resolve(contract ? ContractDao.fromBuffer(contract) : undefined);
  }

  getContracts(): Promise<ContractDao[]> {
    return Promise.resolve(Array.from(this.#contracts.values()).map(c => ContractDao.fromBuffer(c)));
  }
}
