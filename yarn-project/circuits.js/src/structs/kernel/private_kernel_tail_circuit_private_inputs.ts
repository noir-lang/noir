import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import {
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
} from '../../constants.gen.js';
import { countAccumulatedItems } from '../../utils/index.js';
import { NoteLogHash, ScopedEncryptedLogHash, ScopedLogHash } from '../log_hash.js';
import { ScopedNoteHash } from '../note_hash.js';
import { ScopedNullifier } from '../nullifier.js';
import { PrivateKernelData } from './private_kernel_data.js';

export class PrivateKernelTailHints {
  constructor(
    /*
     * The sorted new note hashes.
     */
    public sortedNewNoteHashes: Tuple<ScopedNoteHash, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The sorted new note hashes indexes. Maps original to sorted.
     */
    public sortedNewNoteHashesIndexes: Tuple<number, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The sorted new nullifiers. Maps original to sorted.
     */
    public sortedNewNullifiers: Tuple<ScopedNullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The sorted new nullifiers indexes.
     */
    public sortedNewNullifiersIndexes: Tuple<number, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The sorted encrypted note log hashes.
     */
    public sortedNoteEncryptedLogHashes: Tuple<NoteLogHash, typeof MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
    /**
     * The sorted encrypted note log hashes indexes. Maps original to sorted.
     */
    public sortedNoteEncryptedLogHashesIndexes: Tuple<number, typeof MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
    /**
     * The sorted encrypted log hashes.
     */
    public sortedEncryptedLogHashes: Tuple<ScopedEncryptedLogHash, typeof MAX_ENCRYPTED_LOGS_PER_TX>,
    /**
     * The sorted encrypted log hashes indexes. Maps original to sorted.
     */
    public sortedEncryptedLogHashesIndexes: Tuple<number, typeof MAX_ENCRYPTED_LOGS_PER_TX>,
    /**
     * The sorted unencrypted log hashes.
     */
    public sortedUnencryptedLogHashes: Tuple<ScopedLogHash, typeof MAX_UNENCRYPTED_LOGS_PER_TX>,
    /**
     * The sorted encrypted log hashes indexes. Maps original to sorted.
     */
    public sortedUnencryptedLogHashesIndexes: Tuple<number, typeof MAX_UNENCRYPTED_LOGS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.sortedNewNoteHashes,
      this.sortedNewNoteHashesIndexes,
      this.sortedNewNullifiers,
      this.sortedNewNullifiersIndexes,
      this.sortedNoteEncryptedLogHashes,
      this.sortedNoteEncryptedLogHashesIndexes,
      this.sortedEncryptedLogHashes,
      this.sortedEncryptedLogHashesIndexes,
      this.sortedUnencryptedLogHashes,
      this.sortedUnencryptedLogHashesIndexes,
    );
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelTailHints(
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_TX, ScopedNoteHash),
      reader.readNumbers(MAX_NEW_NOTE_HASHES_PER_TX),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, ScopedNullifier),
      reader.readNumbers(MAX_NEW_NULLIFIERS_PER_TX),
      reader.readArray(MAX_NOTE_ENCRYPTED_LOGS_PER_TX, NoteLogHash),
      reader.readNumbers(MAX_NOTE_ENCRYPTED_LOGS_PER_TX),
      reader.readArray(MAX_ENCRYPTED_LOGS_PER_TX, ScopedEncryptedLogHash),
      reader.readNumbers(MAX_ENCRYPTED_LOGS_PER_TX),
      reader.readArray(MAX_UNENCRYPTED_LOGS_PER_TX, ScopedLogHash),
      reader.readNumbers(MAX_UNENCRYPTED_LOGS_PER_TX),
    );
  }
}

/**
 * Input to the private kernel circuit - tail call.
 */
export class PrivateKernelTailCircuitPrivateInputs {
  constructor(
    /**
     * The previous kernel data
     */
    public previousKernel: PrivateKernelData,
    public hints: PrivateKernelTailHints,
  ) {}

  isForPublic() {
    return countAccumulatedItems(this.previousKernel.publicInputs.end.publicCallStack) > 0;
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.previousKernel, this.hints);
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelTailCircuitPrivateInputs {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelTailCircuitPrivateInputs(
      reader.readObject(PrivateKernelData),
      reader.readObject(PrivateKernelTailHints),
    );
  }
}
