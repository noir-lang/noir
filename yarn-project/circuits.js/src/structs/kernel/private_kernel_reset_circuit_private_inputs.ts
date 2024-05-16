import { GrumpkinScalar } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import {
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
} from '../../constants.gen.js';
import { type GrumpkinPrivateKey } from '../../types/grumpkin_private_key.js';
import { countAccumulatedItems } from '../../utils/index.js';
import { NoteLogHash } from '../log_hash.js';
import { ScopedNoteHash } from '../note_hash.js';
import { ScopedNullifier } from '../nullifier.js';
import {
  type NoteHashReadRequestHints,
  type NullifierReadRequestHints,
  noteHashReadRequestHintsFromBuffer,
  nullifierReadRequestHintsFromBuffer,
} from '../read_request_hints/index.js';
import { PrivateKernelData } from './private_kernel_data.js';

export class PrivateKernelResetOutputs {
  constructor(
    public noteHashes: Tuple<ScopedNoteHash, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    public nullifiers: Tuple<ScopedNullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    public noteEncryptedLogHashes: Tuple<NoteLogHash, typeof MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.noteHashes, this.nullifiers, this.noteEncryptedLogHashes);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelResetOutputs(
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_TX, ScopedNoteHash),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, ScopedNullifier),
      reader.readArray(MAX_NOTE_ENCRYPTED_LOGS_PER_TX, NoteLogHash),
    );
  }
}

export class PrivateKernelResetHints {
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
     * Contains hints for the transient logs to locate corresponding note hashes.
     */
    public transientNoteHashIndexesForLogs: Tuple<number, typeof MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
    /**
     * Contains hints for the transient read requests to localize corresponding commitments.
     */
    public noteHashReadRequestHints: NoteHashReadRequestHints,
    /**
     * Contains hints for the nullifier read requests to locate corresponding pending or settled nullifiers.
     */
    public nullifierReadRequestHints: NullifierReadRequestHints,

    /**
     * The master nullifier secret keys for the nullifier key validation requests.
     */
    public masterNullifierSecretKeys: Tuple<GrumpkinPrivateKey, typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.transientNullifierIndexesForNoteHashes,
      this.transientNoteHashIndexesForNullifiers,
      this.transientNoteHashIndexesForLogs,
      this.noteHashReadRequestHints,
      this.nullifierReadRequestHints,
      this.masterNullifierSecretKeys,
    );
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelResetHints(
      reader.readNumbers(MAX_NEW_NOTE_HASHES_PER_TX),
      reader.readNumbers(MAX_NEW_NULLIFIERS_PER_TX),
      reader.readNumbers(MAX_NOTE_ENCRYPTED_LOGS_PER_TX),
      reader.readObject({ fromBuffer: noteHashReadRequestHintsFromBuffer }),
      reader.readObject({ fromBuffer: nullifierReadRequestHintsFromBuffer }),
      reader.readArray(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, GrumpkinScalar),
    );
  }
}

/**
 * Input to the private kernel circuit - reset call.
 */
export class PrivateKernelResetCircuitPrivateInputs {
  constructor(
    /**
     * The previous kernel data
     */
    public previousKernel: PrivateKernelData,
    public outputs: PrivateKernelResetOutputs,
    public hints: PrivateKernelResetHints,
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
  static fromBuffer(buffer: Buffer | BufferReader): PrivateKernelResetCircuitPrivateInputs {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelResetCircuitPrivateInputs(
      reader.readObject(PrivateKernelData),
      reader.readObject(PrivateKernelResetOutputs),
      reader.readObject(PrivateKernelResetHints),
    );
  }
}
