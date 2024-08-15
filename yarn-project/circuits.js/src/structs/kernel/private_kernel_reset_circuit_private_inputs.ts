import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import { countAccumulatedItems } from '../../utils/index.js';
import {
  KeyValidationHint,
  type NoteHashReadRequestHints,
  type NullifierReadRequestHints,
  noteHashReadRequestHintsFromBuffer,
  nullifierReadRequestHintsFromBuffer,
} from '../read_request_hints/index.js';
import { TransientDataIndexHint } from '../transient_data_index_hint.js';
import { PrivateKernelData } from './private_kernel_data.js';

export { TransientDataIndexHint } from '../transient_data_index_hint.js';

export class PrivateKernelResetHints<
  NH_RR_PENDING extends number,
  NH_RR_SETTLED extends number,
  NLL_RR_PENDING extends number,
  NLL_RR_SETTLED extends number,
  KEY_VALIDATION_REQUESTS extends number,
  NUM_TRANSIENT_DATA_INDEX_HINTS extends number,
> {
  constructor(
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
    /**
     * Contains hints for the transient note hashes to locate corresponding nullifiers.
     */
    public transientDataIndexHints: Tuple<TransientDataIndexHint, NUM_TRANSIENT_DATA_INDEX_HINTS>,
    /**
     * The "final" minRevertibleSideEffectCounter of a tx, to split the data for squashing.
     * Not the minRevertibleSideEffectCounter at the point the reset circuit is run.
     */
    public validationRequestsSplitCounter: number,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.noteHashReadRequestHints,
      this.nullifierReadRequestHints,
      this.keyValidationHints,
      this.transientDataIndexHints,
      this.validationRequestsSplitCounter,
    );
  }

  trimToSizes<
    NEW_NH_RR_PENDING extends number,
    NEW_NH_RR_SETTLED extends number,
    NEW_NLL_RR_PENDING extends number,
    NEW_NLL_RR_SETTLED extends number,
    NEW_KEY_VALIDATION_REQUESTS extends number,
    NUM_TRANSIENT_DATA_INDEX_HINTS extends number,
  >(
    numNoteHashReadRequestPending: NEW_NH_RR_PENDING,
    numNoteHashReadRequestSettled: NEW_NH_RR_SETTLED,
    numNullifierReadRequestPending: NEW_NLL_RR_PENDING,
    numNullifierReadRequestSettled: NEW_NLL_RR_SETTLED,
    numKeyValidationRequests: NEW_KEY_VALIDATION_REQUESTS,
    numTransientDataIndexHints: NUM_TRANSIENT_DATA_INDEX_HINTS,
  ): PrivateKernelResetHints<
    NEW_NH_RR_PENDING,
    NEW_NH_RR_SETTLED,
    NEW_NLL_RR_PENDING,
    NEW_NLL_RR_SETTLED,
    NEW_KEY_VALIDATION_REQUESTS,
    NUM_TRANSIENT_DATA_INDEX_HINTS
  > {
    return new PrivateKernelResetHints(
      this.noteHashReadRequestHints.trimToSizes(numNoteHashReadRequestPending, numNoteHashReadRequestSettled),
      this.nullifierReadRequestHints.trimToSizes(numNullifierReadRequestPending, numNullifierReadRequestSettled),
      this.keyValidationHints.slice(0, numKeyValidationRequests) as Tuple<
        KeyValidationHint,
        NEW_KEY_VALIDATION_REQUESTS
      >,
      this.transientDataIndexHints.slice(0, numTransientDataIndexHints) as Tuple<
        TransientDataIndexHint,
        NUM_TRANSIENT_DATA_INDEX_HINTS
      >,
      this.validationRequestsSplitCounter,
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
    NUM_TRANSIENT_DATA_INDEX_HINTS extends number,
  >(
    buffer: Buffer | BufferReader,
    numNoteHashReadRequestPending: NH_RR_PENDING,
    numNoteHashReadRequestSettled: NH_RR_SETTLED,
    numNullifierReadRequestPending: NLL_RR_PENDING,
    numNullifierReadRequestSettled: NLL_RR_SETTLED,
    numNullifierKeys: KEY_VALIDATION_REQUESTS,
    numTransientDataIndexHints: NUM_TRANSIENT_DATA_INDEX_HINTS,
  ): PrivateKernelResetHints<
    NH_RR_PENDING,
    NH_RR_SETTLED,
    NLL_RR_PENDING,
    NLL_RR_SETTLED,
    KEY_VALIDATION_REQUESTS,
    NUM_TRANSIENT_DATA_INDEX_HINTS
  > {
    const reader = BufferReader.asReader(buffer);
    return new PrivateKernelResetHints(
      reader.readObject({
        fromBuffer: buf =>
          noteHashReadRequestHintsFromBuffer(buf, numNoteHashReadRequestPending, numNoteHashReadRequestSettled),
      }),
      reader.readObject({
        fromBuffer: buf =>
          nullifierReadRequestHintsFromBuffer(buf, numNullifierReadRequestPending, numNullifierReadRequestSettled),
      }),
      reader.readArray(numNullifierKeys, KeyValidationHint),
      reader.readArray(numTransientDataIndexHints, TransientDataIndexHint),
      reader.readNumber(),
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
  NUM_TRANSIENT_DATA_INDEX_HINTS extends number,
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
      KEY_VALIDATION_REQUESTS,
      NUM_TRANSIENT_DATA_INDEX_HINTS
    >,
    public sizeTag: TAG,
  ) {}

  isForPublic() {
    return countAccumulatedItems(this.previousKernel.publicInputs.end.publicCallRequests) > 0;
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
    NUM_TRANSIENT_DATA_INDEX_HINTS extends number,
    TAG extends string,
  >(
    buffer: Buffer | BufferReader,
    numNoteHashReadRequestPending: NH_RR_PENDING,
    numNoteHashReadRequestSettled: NH_RR_SETTLED,
    numNullifierReadRequestPending: NLL_RR_PENDING,
    numNullifierReadRequestSettled: NLL_RR_SETTLED,
    numNullifierKeys: KEY_VALIDATION_REQUESTS,
    numTransientDataIndexHints: NUM_TRANSIENT_DATA_INDEX_HINTS,
    sizeTag: TAG,
  ): PrivateKernelResetCircuitPrivateInputs<
    NH_RR_PENDING,
    NH_RR_SETTLED,
    NLL_RR_PENDING,
    NLL_RR_SETTLED,
    KEY_VALIDATION_REQUESTS,
    NUM_TRANSIENT_DATA_INDEX_HINTS,
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
            numTransientDataIndexHints,
          ),
      }),
      sizeTag,
    );
  }
}
