import { makeTuple } from '@aztec/foundation/array';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';

import {
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_L2_TO_L1_MSGS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_HASHES_PER_TX,
  MAX_NULLIFIERS_PER_TX,
  MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
} from '../../constants.gen.js';
import { ScopedL2ToL1Message } from '../l2_to_l1_message.js';
import { NoteLogHash, ScopedEncryptedLogHash, ScopedLogHash } from '../log_hash.js';
import { ScopedNoteHash } from '../note_hash.js';
import { ScopedNullifier } from '../nullifier.js';
import { PrivateCallRequest } from '../private_call_request.js';
import { PublicCallRequest } from '../public_call_request.js';

/**
 * Specific accumulated data structure for the final ordering private kernel circuit. It is included
 *  in the final public inputs of private kernel circuit.
 */
export class PrivateAccumulatedData {
  constructor(
    /**
     * The new note hashes made in this transaction.
     */
    public noteHashes: Tuple<ScopedNoteHash, typeof MAX_NOTE_HASHES_PER_TX>,
    /**
     * The new nullifiers made in this transaction.
     */
    public nullifiers: Tuple<ScopedNullifier, typeof MAX_NULLIFIERS_PER_TX>,
    /**
     * All the new L2 to L1 messages created in this transaction.
     */
    public l2ToL1Msgs: Tuple<ScopedL2ToL1Message, typeof MAX_L2_TO_L1_MSGS_PER_TX>,
    /**
     * Accumulated encrypted note logs hashes from all the previous kernel iterations.
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public noteEncryptedLogsHashes: Tuple<NoteLogHash, typeof MAX_NOTE_ENCRYPTED_LOGS_PER_TX>,
    /**
     * Accumulated encrypted logs hashes from all the previous kernel iterations.
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public encryptedLogsHashes: Tuple<ScopedEncryptedLogHash, typeof MAX_ENCRYPTED_LOGS_PER_TX>,
    /**
     * Accumulated unencrypted logs hashes from all the previous kernel iterations.
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public unencryptedLogsHashes: Tuple<ScopedLogHash, typeof MAX_UNENCRYPTED_LOGS_PER_TX>,
    /**
     * Accumulated public call requests from all the previous kernel iterations.
     */
    public publicCallRequests: Tuple<PublicCallRequest, typeof MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX>,
    /**
     * Current private call stack.
     */
    public privateCallStack: Tuple<PrivateCallRequest, typeof MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX>,
  ) {}

  toBuffer() {
    return serializeToBuffer(
      this.noteHashes,
      this.nullifiers,
      this.l2ToL1Msgs,
      this.noteEncryptedLogsHashes,
      this.encryptedLogsHashes,
      this.unencryptedLogsHashes,
      this.publicCallRequests,
      this.privateCallStack,
    );
  }

  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns Deserialized object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateAccumulatedData {
    const reader = BufferReader.asReader(buffer);
    return new PrivateAccumulatedData(
      reader.readArray(MAX_NOTE_HASHES_PER_TX, ScopedNoteHash),
      reader.readArray(MAX_NULLIFIERS_PER_TX, ScopedNullifier),
      reader.readArray(MAX_L2_TO_L1_MSGS_PER_TX, ScopedL2ToL1Message),
      reader.readArray(MAX_NOTE_ENCRYPTED_LOGS_PER_TX, NoteLogHash),
      reader.readArray(MAX_ENCRYPTED_LOGS_PER_TX, ScopedEncryptedLogHash),
      reader.readArray(MAX_UNENCRYPTED_LOGS_PER_TX, ScopedLogHash),
      reader.readArray(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, PublicCallRequest),
      reader.readArray(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, PrivateCallRequest),
    );
  }

  /**
   * Deserializes from a string, corresponding to a write in cpp.
   * @param str - String to read from.
   * @returns Deserialized object.
   */
  static fromString(str: string) {
    return PrivateAccumulatedData.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new PrivateAccumulatedData(
      makeTuple(MAX_NOTE_HASHES_PER_TX, ScopedNoteHash.empty),
      makeTuple(MAX_NULLIFIERS_PER_TX, ScopedNullifier.empty),
      makeTuple(MAX_L2_TO_L1_MSGS_PER_TX, ScopedL2ToL1Message.empty),
      makeTuple(MAX_NOTE_ENCRYPTED_LOGS_PER_TX, NoteLogHash.empty),
      makeTuple(MAX_ENCRYPTED_LOGS_PER_TX, ScopedEncryptedLogHash.empty),
      makeTuple(MAX_UNENCRYPTED_LOGS_PER_TX, ScopedLogHash.empty),
      makeTuple(MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX, PublicCallRequest.empty),
      makeTuple(MAX_PRIVATE_CALL_STACK_LENGTH_PER_TX, PrivateCallRequest.empty),
    );
  }
}
