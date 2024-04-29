import { GrumpkinScalar } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import {
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
} from '../../constants.gen.js';
import { type GrumpkinPrivateKey } from '../../types/grumpkin_private_key.js';
import { countAccumulatedItems } from '../../utils/index.js';
import { NoteHashContext } from '../note_hash.js';
import { Nullifier } from '../nullifier.js';
import { type NullifierReadRequestHints, nullifierReadRequestHintsFromBuffer } from '../read_request_hints.js';
import { SideEffect } from '../side_effects.js';
import { PrivateKernelData } from './private_kernel_data.js';

export class PrivateKernelTailOutputs {
  constructor(
    public noteHashes: Tuple<NoteHashContext, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    public nullifiers: Tuple<Nullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.noteHashes, this.nullifiers);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelTailOutputs(
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_TX, NoteHashContext),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, Nullifier),
    );
  }
}

export class PrivateKernelTailHints {
  constructor(
    /**
     * Contains hints for the transient note hashes to locate corresponding nullifiers.
     */
    public transientNullifierIndexesForNoteHashes: Tuple<number, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * Contains hints for the transient nullifiers to locate corresponding note hashes.
     */
    public transientNoteHashIndexesForNullifiers: Tuple<number, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * Contains hints for the transient read requests to localize corresponding commitments.
     */
    public noteHashReadRequestHints: Tuple<number, typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
    /**
     * Contains hints for the nullifier read requests to locate corresponding pending or settled nullifiers.
     */
    public nullifierReadRequestHints: NullifierReadRequestHints,

    /**
     * The master nullifier secret keys for the nullifier key validation requests.
     */
    public masterNullifierSecretKeys: Tuple<GrumpkinPrivateKey, typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX>,
    /*
     * The sorted new note hashes.
     */
    public sortedNewNoteHashes: Tuple<NoteHashContext, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The sorted new note hashes indexes. Maps original to sorted.
     */
    public sortedNewNoteHashesIndexes: Tuple<number, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The sorted new nullifiers. Maps original to sorted.
     */
    public sortedNewNullifiers: Tuple<Nullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The sorted new nullifiers indexes.
     */
    public sortedNewNullifiersIndexes: Tuple<number, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The sorted encrypted log hashes.
     */
    public sortedEncryptedLogHashes: Tuple<SideEffect, typeof MAX_ENCRYPTED_LOGS_PER_TX>,
    /**
     * The sorted encrypted log hashes indexes. Maps original to sorted.
     */
    public sortedEncryptedLogHashesIndexes: Tuple<number, typeof MAX_ENCRYPTED_LOGS_PER_TX>,
    /**
     * The sorted unencrypted log hashes.
     */
    public sortedUnencryptedLogHashes: Tuple<SideEffect, typeof MAX_UNENCRYPTED_LOGS_PER_TX>,
    /**
     * The sorted encrypted log hashes indexes. Maps original to sorted.
     */
    public sortedUnencryptedLogHashesIndexes: Tuple<number, typeof MAX_UNENCRYPTED_LOGS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.transientNullifierIndexesForNoteHashes,
      this.transientNoteHashIndexesForNullifiers,
      this.noteHashReadRequestHints,
      this.nullifierReadRequestHints,
      this.masterNullifierSecretKeys,
      this.sortedNewNoteHashes,
      this.sortedNewNoteHashesIndexes,
      this.sortedNewNullifiers,
      this.sortedNewNullifiersIndexes,
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
      reader.readNumbers(MAX_NEW_NOTE_HASHES_PER_TX),
      reader.readNumbers(MAX_NEW_NULLIFIERS_PER_TX),
      reader.readNumbers(MAX_NOTE_HASH_READ_REQUESTS_PER_TX),
      reader.readObject({ fromBuffer: nullifierReadRequestHintsFromBuffer }),
      reader.readArray(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, GrumpkinScalar),
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_TX, NoteHashContext),
      reader.readNumbers(MAX_NEW_NOTE_HASHES_PER_TX),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, Nullifier),
      reader.readNumbers(MAX_NEW_NULLIFIERS_PER_TX),
      reader.readArray(MAX_ENCRYPTED_LOGS_PER_TX, SideEffect),
      reader.readNumbers(MAX_ENCRYPTED_LOGS_PER_TX),
      reader.readArray(MAX_UNENCRYPTED_LOGS_PER_TX, SideEffect),
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
    public outputs: PrivateKernelTailOutputs,
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
    return serializeToBuffer(this.previousKernel, this.outputs, this.hints);
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
      reader.readObject(PrivateKernelTailOutputs),
      reader.readObject(PrivateKernelTailHints),
    );
  }
}
