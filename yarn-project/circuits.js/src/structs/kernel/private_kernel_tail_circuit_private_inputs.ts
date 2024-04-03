import { Fr, GrumpkinScalar } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import {
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
} from '../../constants.gen.js';
import { type GrumpkinPrivateKey } from '../../types/grumpkin_private_key.js';
import { countAccumulatedItems } from '../../utils/index.js';
import { type NullifierReadRequestHints, nullifierReadRequestHintsFromBuffer } from '../read_request_hints.js';
import { SideEffect, SideEffectLinkedToNoteHash } from '../side_effects.js';
import { PrivateKernelData } from './private_kernel_data.js';

/**
 * Input to the private kernel circuit - tail call.
 */
export class PrivateKernelTailCircuitPrivateInputs {
  constructor(
    /**
     * The previous kernel data
     */
    public previousKernel: PrivateKernelData,
    /**
     * The sorted new note hashes.
     */
    public sortedNewNoteHashes: Tuple<SideEffect, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * The sorted new note hashes indexes. Maps original to sorted.
     */
    public sortedNewNoteHashesIndexes: Tuple<number, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
    /**
     * Contains hints for the transient read requests to localize corresponding commitments.
     */
    public readCommitmentHints: Tuple<Fr, typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
    /**
     * The sorted new nullifiers. Maps original to sorted.
     */
    public sortedNewNullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The sorted new nullifiers indexes.
     */
    public sortedNewNullifiersIndexes: Tuple<number, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * Contains hints for the nullifier read requests to locate corresponding pending or settled nullifiers.
     */
    public nullifierReadRequestHints: NullifierReadRequestHints,
    /**
     * Contains hints for the transient nullifiers to localize corresponding commitments.
     */
    public nullifierCommitmentHints: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    /**
     * The master nullifier secret keys for the nullifier key validation requests.
     */
    public masterNullifierSecretKeys: Tuple<GrumpkinPrivateKey, typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX>,
  ) {}

  isForPublic() {
    return countAccumulatedItems(this.previousKernel.publicInputs.end.publicCallStack) > 0;
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.previousKernel,
      this.sortedNewNoteHashes,
      this.sortedNewNoteHashesIndexes,
      this.readCommitmentHints,
      this.sortedNewNullifiers,
      this.sortedNewNullifiersIndexes,
      this.nullifierReadRequestHints,
      this.nullifierCommitmentHints,
      this.masterNullifierSecretKeys,
    );
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
      reader.readArray(MAX_NEW_NOTE_HASHES_PER_TX, SideEffect),
      reader.readNumbers(MAX_NEW_NOTE_HASHES_PER_TX),
      reader.readArray(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, Fr),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, SideEffectLinkedToNoteHash),
      reader.readNumbers(MAX_NEW_NULLIFIERS_PER_TX),
      reader.readObject({ fromBuffer: nullifierReadRequestHintsFromBuffer }),
      reader.readArray(MAX_NEW_NULLIFIERS_PER_TX, Fr),
      reader.readArray(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, GrumpkinScalar),
    );
  }
}
