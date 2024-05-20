import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import {
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
} from '../../constants.gen.js';
import { countAccumulatedItems } from '../../utils/index.js';
import { NoteLogHash } from '../log_hash.js';
import { ScopedNoteHash } from '../note_hash.js';
import { ScopedNullifier } from '../nullifier.js';
import {
  type NoteHashReadRequestHints,
  NullifierKeyHint,
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

export class PrivateKernelResetHints<
  NH_RR_PENDING extends number,
  NH_RR_SETTLED extends number,
  NLL_RR_PENDING extends number,
  NLL_RR_SETTLED extends number,
  NLL_KEYS extends number,
> {
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
    public noteHashReadRequestHints: NoteHashReadRequestHints<NH_RR_PENDING, NH_RR_SETTLED>,
    /**
     * Contains hints for the nullifier read requests to locate corresponding pending or settled nullifiers.
     */
    public nullifierReadRequestHints: NullifierReadRequestHints<NLL_RR_PENDING, NLL_RR_SETTLED>,

    /**
     * The master nullifier secret keys for the nullifier key validation requests.
     */
    public masterNullifierSecretKeys: Tuple<NullifierKeyHint, NLL_KEYS>,
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

  trimToSizes<
    NEW_NH_RR_PENDING extends number,
    NEW_NH_RR_SETTLED extends number,
    NEW_NLL_RR_PENDING extends number,
    NEW_NLL_RR_SETTLED extends number,
    NEW_NLL_KEYS extends number,
  >(
    numNoteHashReadRequestPending: NEW_NH_RR_PENDING,
    numNoteHashReadRequestSettled: NEW_NH_RR_SETTLED,
    numNullifierReadRequestPending: NEW_NLL_RR_PENDING,
    numNullifierReadRequestSettled: NEW_NLL_RR_SETTLED,
    numNullifierKeys: NEW_NLL_KEYS,
  ): PrivateKernelResetHints<
    NEW_NH_RR_PENDING,
    NEW_NH_RR_SETTLED,
    NEW_NLL_RR_PENDING,
    NEW_NLL_RR_SETTLED,
    NEW_NLL_KEYS
  > {
    return new PrivateKernelResetHints(
      this.transientNullifierIndexesForNoteHashes,
      this.transientNoteHashIndexesForNullifiers,
      this.transientNoteHashIndexesForLogs,
      this.noteHashReadRequestHints.trimToSizes(numNoteHashReadRequestPending, numNoteHashReadRequestSettled),
      this.nullifierReadRequestHints.trimToSizes(numNullifierReadRequestPending, numNullifierReadRequestSettled),
      this.masterNullifierSecretKeys.slice(0, numNullifierKeys) as Tuple<NullifierKeyHint, NEW_NLL_KEYS>,
    );
  }
  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer<
    NH_RR_PENDING extends number,
    NH_RR_SETTLED extends number,
    NLL_RR_PENDING extends number,
    NLL_RR_SETTLED extends number,
    NLL_KEYS extends number,
  >(
    buffer: Buffer | BufferReader,
    numNoteHashReadRequestPending: NH_RR_PENDING,
    numNoteHashReadRequestSettled: NH_RR_SETTLED,
    numNullifierReadRequestPending: NLL_RR_PENDING,
    numNullifierReadRequestSettled: NLL_RR_SETTLED,
    numNullifierKeys: NLL_KEYS,
  ): PrivateKernelResetHints<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, NLL_KEYS> {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelResetHints(
      reader.readNumbers(MAX_NEW_NOTE_HASHES_PER_TX),
      reader.readNumbers(MAX_NEW_NULLIFIERS_PER_TX),
      reader.readNumbers(MAX_NOTE_ENCRYPTED_LOGS_PER_TX),
      reader.readObject({
        fromBuffer: buf =>
          noteHashReadRequestHintsFromBuffer(buf, numNoteHashReadRequestPending, numNoteHashReadRequestSettled),
      }),
      reader.readObject({
        fromBuffer: buf =>
          nullifierReadRequestHintsFromBuffer(buf, numNullifierReadRequestPending, numNullifierReadRequestSettled),
      }),
      reader.readArray(numNullifierKeys, NullifierKeyHint),
    );
  }
}

/**
 * Input to the private kernel circuit - reset call.
 */
export class PrivateKernelResetCircuitPrivateInputs<
  NH_RR_PENDING extends number,
  NH_RR_SETTLED extends number,
  NLL_RR_PENDING extends number,
  NLL_RR_SETTLED extends number,
  NLL_KEYS extends number,
  TAG extends string,
> {
  constructor(
    /**
     * The previous kernel data
     */
    public previousKernel: PrivateKernelData,
    public outputs: PrivateKernelResetOutputs,
    public hints: PrivateKernelResetHints<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, NLL_KEYS>,
    public sizeTag: TAG,
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
  static fromBuffer<
    NH_RR_PENDING extends number,
    NH_RR_SETTLED extends number,
    NLL_RR_PENDING extends number,
    NLL_RR_SETTLED extends number,
    NLL_KEYS extends number,
    TAG extends string,
  >(
    buffer: Buffer | BufferReader,
    numNoteHashReadRequestPending: NH_RR_PENDING,
    numNoteHashReadRequestSettled: NH_RR_SETTLED,
    numNullifierReadRequestPending: NLL_RR_PENDING,
    numNullifierReadRequestSettled: NLL_RR_SETTLED,
    numNullifierKeys: NLL_KEYS,
    sizeTag: TAG,
  ): PrivateKernelResetCircuitPrivateInputs<
    NH_RR_PENDING,
    NH_RR_SETTLED,
    NLL_RR_PENDING,
    NLL_RR_SETTLED,
    NLL_KEYS,
    TAG
  > {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelResetCircuitPrivateInputs(
      reader.readObject(PrivateKernelData),
      reader.readObject(PrivateKernelResetOutputs),
      reader.readObject({
        fromBuffer: buf =>
          PrivateKernelResetHints.fromBuffer(
            buf,
            numNoteHashReadRequestPending,
            numNoteHashReadRequestSettled,
            numNullifierReadRequestPending,
            numNullifierReadRequestSettled,
            numNullifierKeys,
          ),
      }),
      sizeTag,
    );
  }
}
