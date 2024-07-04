import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { MAX_NOTE_HASHES_PER_TX, MAX_NULLIFIERS_PER_TX } from '../../constants.gen.js';
import { countAccumulatedItems } from '../../utils/index.js';
import {
  KeyValidationHint,
  type NoteHashReadRequestHints,
  type NullifierReadRequestHints,
  noteHashReadRequestHintsFromBuffer,
  nullifierReadRequestHintsFromBuffer,
} from '../read_request_hints/index.js';
import { PrivateKernelData } from './private_kernel_data.js';

export class PrivateKernelResetHints<
  NH_RR_PENDING extends number,
  NH_RR_SETTLED extends number,
  NLL_RR_PENDING extends number,
  NLL_RR_SETTLED extends number,
  KEY_VALIDATION_REQUESTS extends number,
> {
  constructor(
    /**
     * Contains hints for the transient note hashes to locate corresponding nullifiers.
     */
    public transientNullifierIndexesForNoteHashes: Tuple<number, typeof MAX_NOTE_HASHES_PER_TX>,
    /**
     * Contains hints for the transient nullifiers to locate corresponding note hashes.
     */
    public transientNoteHashIndexesForNullifiers: Tuple<number, typeof MAX_NULLIFIERS_PER_TX>,
    /**
     * Contains hints for the transient read requests to localize corresponding commitments.
     */
    public noteHashReadRequestHints: NoteHashReadRequestHints<NH_RR_PENDING, NH_RR_SETTLED>,
    /**
     * Contains hints for the nullifier read requests to locate corresponding pending or settled nullifiers.
     */
    public nullifierReadRequestHints: NullifierReadRequestHints<NLL_RR_PENDING, NLL_RR_SETTLED>,
    /**
     * Contains hints for key validation request.
     */
    public keyValidationHints: Tuple<KeyValidationHint, KEY_VALIDATION_REQUESTS>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.transientNullifierIndexesForNoteHashes,
      this.transientNoteHashIndexesForNullifiers,
      this.noteHashReadRequestHints,
      this.nullifierReadRequestHints,
      this.keyValidationHints,
    );
  }

  trimToSizes<
    NEW_NH_RR_PENDING extends number,
    NEW_NH_RR_SETTLED extends number,
    NEW_NLL_RR_PENDING extends number,
    NEW_NLL_RR_SETTLED extends number,
    NEW_KEY_VALIDATION_REQUESTS extends number,
  >(
    numNoteHashReadRequestPending: NEW_NH_RR_PENDING,
    numNoteHashReadRequestSettled: NEW_NH_RR_SETTLED,
    numNullifierReadRequestPending: NEW_NLL_RR_PENDING,
    numNullifierReadRequestSettled: NEW_NLL_RR_SETTLED,
    numKeyValidationRequests: NEW_KEY_VALIDATION_REQUESTS,
  ): PrivateKernelResetHints<
    NEW_NH_RR_PENDING,
    NEW_NH_RR_SETTLED,
    NEW_NLL_RR_PENDING,
    NEW_NLL_RR_SETTLED,
    NEW_KEY_VALIDATION_REQUESTS
  > {
    return new PrivateKernelResetHints(
      this.transientNullifierIndexesForNoteHashes,
      this.transientNoteHashIndexesForNullifiers,
      this.noteHashReadRequestHints.trimToSizes(numNoteHashReadRequestPending, numNoteHashReadRequestSettled),
      this.nullifierReadRequestHints.trimToSizes(numNullifierReadRequestPending, numNullifierReadRequestSettled),
      this.keyValidationHints.slice(0, numKeyValidationRequests) as Tuple<
        KeyValidationHint,
        NEW_KEY_VALIDATION_REQUESTS
      >,
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
    KEY_VALIDATION_REQUESTS extends number,
  >(
    buffer: Buffer | BufferReader,
    numNoteHashReadRequestPending: NH_RR_PENDING,
    numNoteHashReadRequestSettled: NH_RR_SETTLED,
    numNullifierReadRequestPending: NLL_RR_PENDING,
    numNullifierReadRequestSettled: NLL_RR_SETTLED,
    numNullifierKeys: KEY_VALIDATION_REQUESTS,
  ): PrivateKernelResetHints<NH_RR_PENDING, NH_RR_SETTLED, NLL_RR_PENDING, NLL_RR_SETTLED, KEY_VALIDATION_REQUESTS> {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelResetHints(
      reader.readNumbers(MAX_NOTE_HASHES_PER_TX),
      reader.readNumbers(MAX_NULLIFIERS_PER_TX),
      reader.readObject({
        fromBuffer: buf =>
          noteHashReadRequestHintsFromBuffer(buf, numNoteHashReadRequestPending, numNoteHashReadRequestSettled),
      }),
      reader.readObject({
        fromBuffer: buf =>
          nullifierReadRequestHintsFromBuffer(buf, numNullifierReadRequestPending, numNullifierReadRequestSettled),
      }),
      reader.readArray(numNullifierKeys, KeyValidationHint),
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
  KEY_VALIDATION_REQUESTS extends number,
  TAG extends string,
> {
  constructor(
    /**
     * The previous kernel data
     */
    public previousKernel: PrivateKernelData,
    public hints: PrivateKernelResetHints<
      NH_RR_PENDING,
      NH_RR_SETTLED,
      NLL_RR_PENDING,
      NLL_RR_SETTLED,
      KEY_VALIDATION_REQUESTS
    >,
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
    return serializeToBuffer(this.previousKernel, this.hints);
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
    KEY_VALIDATION_REQUESTS extends number,
    TAG extends string,
  >(
    buffer: Buffer | BufferReader,
    numNoteHashReadRequestPending: NH_RR_PENDING,
    numNoteHashReadRequestSettled: NH_RR_SETTLED,
    numNullifierReadRequestPending: NLL_RR_PENDING,
    numNullifierReadRequestSettled: NLL_RR_SETTLED,
    numNullifierKeys: KEY_VALIDATION_REQUESTS,
    sizeTag: TAG,
  ): PrivateKernelResetCircuitPrivateInputs<
    NH_RR_PENDING,
    NH_RR_SETTLED,
    NLL_RR_PENDING,
    NLL_RR_SETTLED,
    KEY_VALIDATION_REQUESTS,
    TAG
  > {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelResetCircuitPrivateInputs(
      reader.readObject(PrivateKernelData),
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
