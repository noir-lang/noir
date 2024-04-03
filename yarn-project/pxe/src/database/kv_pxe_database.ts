import { MerkleTreeId, type NoteFilter, NoteStatus, type PublicKey } from '@aztec/circuit-types';
import { AztecAddress, CompleteAddress, Header } from '@aztec/circuits.js';
import { type ContractArtifact } from '@aztec/foundation/abi';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { Fr, type Point } from '@aztec/foundation/fields';
import {
  type AztecArray,
  type AztecKVStore,
  type AztecMap,
  type AztecMultiMap,
  type AztecSingleton,
} from '@aztec/kv-store';
import { contractArtifactFromBuffer, contractArtifactToBuffer } from '@aztec/types/abi';
import { type ContractInstanceWithAddress, SerializableContractInstance } from '@aztec/types/contracts';

import { DeferredNoteDao } from './deferred_note_dao.js';
import { NoteDao } from './note_dao.js';
import { type PxeDatabase } from './pxe_database.js';

/**
 * A PXE database backed by LMDB.
 */
export class KVPxeDatabase implements PxeDatabase {
  #synchronizedBlock: AztecSingleton<Buffer>;
  #addresses: AztecArray<Buffer>;
  #addressIndex: AztecMap<string, number>;
  #authWitnesses: AztecMap<string, Buffer[]>;
  #capsules: AztecArray<Buffer[]>;
  #notes: AztecMap<string, Buffer>;
  #nullifiedNotes: AztecMap<string, Buffer>;
  #nullifierToNoteId: AztecMap<string, string>;
  #notesByContract: AztecMultiMap<string, string>;
  #notesByStorageSlot: AztecMultiMap<string, string>;
  #notesByTxHash: AztecMultiMap<string, string>;
  #notesByOwner: AztecMultiMap<string, string>;
  #nullifiedNotesByContract: AztecMultiMap<string, string>;
  #nullifiedNotesByStorageSlot: AztecMultiMap<string, string>;
  #nullifiedNotesByTxHash: AztecMultiMap<string, string>;
  #nullifiedNotesByOwner: AztecMultiMap<string, string>;
  #deferredNotes: AztecArray<Buffer | null>;
  #deferredNotesByContract: AztecMultiMap<string, number>;
  #syncedBlockPerPublicKey: AztecMap<string, number>;
  #contractArtifacts: AztecMap<string, Buffer>;
  #contractInstances: AztecMap<string, Buffer>;
  #db: AztecKVStore;

  constructor(private db: AztecKVStore) {
    this.#db = db;

    this.#addresses = db.openArray('addresses');
    this.#addressIndex = db.openMap('address_index');

    this.#authWitnesses = db.openMap('auth_witnesses');
    this.#capsules = db.openArray('capsules');

    this.#contractArtifacts = db.openMap('contract_artifacts');
    this.#contractInstances = db.openMap('contracts_instances');

    this.#synchronizedBlock = db.openSingleton('header');
    this.#syncedBlockPerPublicKey = db.openMap('synced_block_per_public_key');

    this.#notes = db.openMap('notes');
    this.#nullifiedNotes = db.openMap('nullified_notes');
    this.#nullifierToNoteId = db.openMap('nullifier_to_note');

    this.#notesByContract = db.openMultiMap('notes_by_contract');
    this.#notesByStorageSlot = db.openMultiMap('notes_by_storage_slot');
    this.#notesByTxHash = db.openMultiMap('notes_by_tx_hash');
    this.#notesByOwner = db.openMultiMap('notes_by_owner');

    this.#nullifiedNotesByContract = db.openMultiMap('nullified_notes_by_contract');
    this.#nullifiedNotesByStorageSlot = db.openMultiMap('nullified_notes_by_storage_slot');
    this.#nullifiedNotesByTxHash = db.openMultiMap('nullified_notes_by_tx_hash');
    this.#nullifiedNotesByOwner = db.openMultiMap('nullified_notes_by_owner');

    this.#deferredNotes = db.openArray('deferred_notes');
    this.#deferredNotesByContract = db.openMultiMap('deferred_notes_by_contract');
  }

  public async getContract(
    address: AztecAddress,
  ): Promise<(ContractInstanceWithAddress & ContractArtifact) | undefined> {
    const instance = await this.getContractInstance(address);
    const artifact = instance && (await this.getContractArtifact(instance?.contractClassId));
    if (!instance || !artifact) {
      return undefined;
    }
    return { ...instance, ...artifact };
  }

  public async addContractArtifact(id: Fr, contract: ContractArtifact): Promise<void> {
    await this.#contractArtifacts.set(id.toString(), contractArtifactToBuffer(contract));
  }

  public getContractArtifact(id: Fr): Promise<ContractArtifact | undefined> {
    const contract = this.#contractArtifacts.get(id.toString());
    // TODO(@spalladino): AztecMap lies and returns Uint8Arrays instead of Buffers, hence the extra Buffer.from.
    return Promise.resolve(contract && contractArtifactFromBuffer(Buffer.from(contract)));
  }

  async addContractInstance(contract: ContractInstanceWithAddress): Promise<void> {
    await this.#contractInstances.set(
      contract.address.toString(),
      new SerializableContractInstance(contract).toBuffer(),
    );
  }

  getContractInstance(address: AztecAddress): Promise<ContractInstanceWithAddress | undefined> {
    const contract = this.#contractInstances.get(address.toString());
    return Promise.resolve(contract && SerializableContractInstance.fromBuffer(contract).withAddress(address));
  }

  getContractsAddresses(): Promise<AztecAddress[]> {
    return Promise.resolve(Array.from(this.#contractInstances.keys()).map(AztecAddress.fromString));
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

  addNotes(notes: NoteDao[]): Promise<void> {
    return this.db.transaction(() => {
      for (const dao of notes) {
        // store notes by their index in the notes hash tree
        // this provides the uniqueness we need to store individual notes
        // and should also return notes in the order that they were created.
        // Had we stored them by their nullifier, they would be returned in random order
        const noteIndex = toBufferBE(dao.index, 32).toString('hex');
        void this.#notes.set(noteIndex, dao.toBuffer());
        void this.#nullifierToNoteId.set(dao.siloedNullifier.toString(), noteIndex);
        void this.#notesByContract.set(dao.contractAddress.toString(), noteIndex);
        void this.#notesByStorageSlot.set(dao.storageSlot.toString(), noteIndex);
        void this.#notesByTxHash.set(dao.txHash.toString(), noteIndex);
        void this.#notesByOwner.set(dao.publicKey.toString(), noteIndex);
      }
    });
  }

  async addDeferredNotes(deferredNotes: DeferredNoteDao[]): Promise<void> {
    const newLength = await this.#deferredNotes.push(...deferredNotes.map(note => note.toBuffer()));
    for (const [index, note] of deferredNotes.entries()) {
      const noteId = newLength - deferredNotes.length + index;
      await this.#deferredNotesByContract.set(note.contractAddress.toString(), noteId);
    }
  }

  getDeferredNotesByContract(contractAddress: AztecAddress): Promise<DeferredNoteDao[]> {
    const noteIds = this.#deferredNotesByContract.getValues(contractAddress.toString());
    const notes: DeferredNoteDao[] = [];
    for (const noteId of noteIds) {
      const serializedNote = this.#deferredNotes.at(noteId);
      if (!serializedNote) {
        continue;
      }

      const note = DeferredNoteDao.fromBuffer(serializedNote);
      notes.push(note);
    }

    return Promise.resolve(notes);
  }

  /**
   * Removes all deferred notes for a given contract address.
   * @param contractAddress - the contract address to remove deferred notes for
   * @returns an array of the removed deferred notes
   */
  removeDeferredNotesByContract(contractAddress: AztecAddress): Promise<DeferredNoteDao[]> {
    return this.#db.transaction(() => {
      const deferredNotes: DeferredNoteDao[] = [];
      const indices = this.#deferredNotesByContract.getValues(contractAddress.toString());

      for (const index of indices) {
        const deferredNoteBuffer = this.#deferredNotes.at(index);
        if (!deferredNoteBuffer) {
          continue;
        } else {
          deferredNotes.push(DeferredNoteDao.fromBuffer(deferredNoteBuffer));
        }

        void this.#deferredNotesByContract.deleteValue(contractAddress.toString(), index);
        void this.#deferredNotes.setAt(index, null);
      }

      return deferredNotes;
    });
  }

  #getNotes(filter: NoteFilter): NoteDao[] {
    const publicKey: PublicKey | undefined = filter.owner
      ? this.#getCompleteAddress(filter.owner)?.publicKey
      : undefined;

    filter.status = filter.status ?? NoteStatus.ACTIVE;

    const candidateNoteSources = [];

    candidateNoteSources.push({
      ids: publicKey
        ? this.#notesByOwner.getValues(publicKey.toString())
        : filter.txHash
        ? this.#notesByTxHash.getValues(filter.txHash.toString())
        : filter.contractAddress
        ? this.#notesByContract.getValues(filter.contractAddress.toString())
        : filter.storageSlot
        ? this.#notesByStorageSlot.getValues(filter.storageSlot.toString())
        : this.#notes.keys(),
      notes: this.#notes,
    });

    if (filter.status == NoteStatus.ACTIVE_OR_NULLIFIED) {
      candidateNoteSources.push({
        ids: publicKey
          ? this.#nullifiedNotesByOwner.getValues(publicKey.toString())
          : filter.txHash
          ? this.#nullifiedNotesByTxHash.getValues(filter.txHash.toString())
          : filter.contractAddress
          ? this.#nullifiedNotesByContract.getValues(filter.contractAddress.toString())
          : filter.storageSlot
          ? this.#nullifiedNotesByStorageSlot.getValues(filter.storageSlot.toString())
          : this.#nullifiedNotes.keys(),
        notes: this.#nullifiedNotes,
      });
    }

    const result: NoteDao[] = [];
    for (const { ids, notes } of candidateNoteSources) {
      for (const id of ids) {
        const serializedNote = notes.get(id);
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
    }

    return result;
  }

  getNotes(filter: NoteFilter): Promise<NoteDao[]> {
    return Promise.resolve(this.#getNotes(filter));
  }

  removeNullifiedNotes(nullifiers: Fr[], account: PublicKey): Promise<NoteDao[]> {
    if (nullifiers.length === 0) {
      return Promise.resolve([]);
    }

    return this.#db.transaction(() => {
      const nullifiedNotes: NoteDao[] = [];

      for (const nullifier of nullifiers) {
        const noteIndex = this.#nullifierToNoteId.get(nullifier.toString());
        if (!noteIndex) {
          continue;
        }

        const noteBuffer = noteIndex ? this.#notes.get(noteIndex) : undefined;

        if (!noteBuffer) {
          // note doesn't exist. Maybe it got nullified already
          continue;
        }

        const note = NoteDao.fromBuffer(noteBuffer);
        if (!note.publicKey.equals(account)) {
          // tried to nullify someone else's note
          continue;
        }

        nullifiedNotes.push(note);

        void this.#notes.delete(noteIndex);
        void this.#notesByOwner.deleteValue(account.toString(), noteIndex);
        void this.#notesByTxHash.deleteValue(note.txHash.toString(), noteIndex);
        void this.#notesByContract.deleteValue(note.contractAddress.toString(), noteIndex);
        void this.#notesByStorageSlot.deleteValue(note.storageSlot.toString(), noteIndex);

        void this.#nullifiedNotes.set(noteIndex, note.toBuffer());
        void this.#nullifiedNotesByContract.set(note.contractAddress.toString(), noteIndex);
        void this.#nullifiedNotesByStorageSlot.set(note.storageSlot.toString(), noteIndex);
        void this.#nullifiedNotesByTxHash.set(note.txHash.toString(), noteIndex);
        void this.#nullifiedNotesByOwner.set(note.publicKey.toString(), noteIndex);

        void this.#nullifierToNoteId.delete(nullifier.toString());
      }

      return nullifiedNotes;
    });
  }

  async setHeader(header: Header): Promise<void> {
    await this.#synchronizedBlock.set(header.toBuffer());
  }

  getBlockNumber(): number | undefined {
    const headerBuffer = this.#synchronizedBlock.get();
    if (!headerBuffer) {
      return undefined;
    }

    return Number(Header.fromBuffer(headerBuffer).globalVariables.blockNumber.toBigInt());
  }

  getHeader(): Header {
    const headerBuffer = this.#synchronizedBlock.get();
    if (!headerBuffer) {
      throw new Error(`Header not set`);
    }

    return Header.fromBuffer(headerBuffer);
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

  #getCompleteAddress(address: AztecAddress): CompleteAddress | undefined {
    const index = this.#addressIndex.get(address.toString());
    if (typeof index === 'undefined') {
      return undefined;
    }

    const value = this.#addresses.at(index);
    return value ? CompleteAddress.fromBuffer(value) : undefined;
  }

  getCompleteAddress(address: AztecAddress): Promise<CompleteAddress | undefined> {
    return Promise.resolve(this.#getCompleteAddress(address));
  }

  getCompleteAddresses(): Promise<CompleteAddress[]> {
    return Promise.resolve(Array.from(this.#addresses).map(v => CompleteAddress.fromBuffer(v)));
  }

  getSynchedBlockNumberForPublicKey(publicKey: Point): number | undefined {
    return this.#syncedBlockPerPublicKey.get(publicKey.toString());
  }

  setSynchedBlockNumberForPublicKey(publicKey: Point, blockNumber: number): Promise<void> {
    return this.#syncedBlockPerPublicKey.set(publicKey.toString(), blockNumber);
  }

  estimateSize(): number {
    const notesSize = Array.from(this.#getNotes({})).reduce((sum, note) => sum + note.getSize(), 0);
    const authWitsSize = Array.from(this.#authWitnesses.values()).reduce(
      (sum, value) => sum + value.length * Fr.SIZE_IN_BYTES,
      0,
    );
    const addressesSize = this.#addresses.length * CompleteAddress.SIZE_IN_BYTES;
    const treeRootsSize = Object.keys(MerkleTreeId).length * Fr.SIZE_IN_BYTES;

    return notesSize + treeRootsSize + authWitsSize + addressesSize;
  }
}
