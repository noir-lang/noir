import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';

import {
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_L2_TO_L1_MSGS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_NOTE_HASHES_PER_TX,
  MAX_NULLIFIERS_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
} from '../../constants.gen.js';
import { Gas } from '../gas.js';
import { LogHash, ScopedLogHash } from '../log_hash.js';
import { NoteHash } from '../note_hash.js';
import { Nullifier } from '../nullifier.js';
import { PublicCallRequest } from '../public_call_request.js';
import { PublicDataUpdateRequest } from '../public_data_update_request.js';
import { PublicAccumulatedData } from './public_accumulated_data.js';

/**
 * TESTS-ONLY CLASS
 * Builder for PublicAccumulatedData, used to conveniently construct instances for testing,
 * as PublicAccumulatedData is (or will shortly be) immutable.
 *
 */
export class PublicAccumulatedDataBuilder {
  private noteHashes: NoteHash[] = [];
  private nullifiers: Nullifier[] = [];
  private l2ToL1Msgs: Fr[] = [];
  private noteEncryptedLogsHashes: LogHash[] = [];
  private encryptedLogsHashes: LogHash[] = [];
  private unencryptedLogsHashes: ScopedLogHash[] = [];
  private publicDataUpdateRequests: PublicDataUpdateRequest[] = [];
  private publicCallStack: PublicCallRequest[] = [];
  private gasUsed: Gas = Gas.empty();

  pushNoteHash(newNoteHash: NoteHash) {
    this.noteHashes.push(newNoteHash);
    return this;
  }

  withNoteHashes(noteHashes: NoteHash[]) {
    this.noteHashes = noteHashes;
    return this;
  }

  pushNullifier(newNullifier: Nullifier) {
    this.nullifiers.push(newNullifier);
    return this;
  }

  withNullifiers(nullifiers: Nullifier[]) {
    this.nullifiers = nullifiers;
    return this;
  }

  pushL2ToL1Msg(newL2ToL1Msg: Fr) {
    this.l2ToL1Msgs.push(newL2ToL1Msg);
    return this;
  }

  withL2ToL1Msgs(l2ToL1Msgs: Fr[]) {
    this.l2ToL1Msgs = l2ToL1Msgs;
    return this;
  }

  pushNoteEncryptedLogsHash(noteEncryptedLogsHash: LogHash) {
    this.noteEncryptedLogsHashes.push(noteEncryptedLogsHash);
    return this;
  }

  withNoteEncryptedLogsHashes(noteEncryptedLogsHashes: LogHash[]) {
    this.noteEncryptedLogsHashes = noteEncryptedLogsHashes;
    return this;
  }

  pushEncryptedLogsHash(encryptedLogsHash: LogHash) {
    this.encryptedLogsHashes.push(encryptedLogsHash);
    return this;
  }

  withEncryptedLogsHashes(encryptedLogsHashes: LogHash[]) {
    this.encryptedLogsHashes = encryptedLogsHashes;
    return this;
  }

  pushUnencryptedLogsHash(unencryptedLogsHash: ScopedLogHash) {
    this.unencryptedLogsHashes.push(unencryptedLogsHash);
    return this;
  }

  withUnencryptedLogsHashes(unencryptedLogsHashes: ScopedLogHash[]) {
    this.unencryptedLogsHashes = unencryptedLogsHashes;
    return this;
  }

  pushPublicDataUpdateRequest(publicDataUpdateRequest: PublicDataUpdateRequest) {
    this.publicDataUpdateRequests.push(publicDataUpdateRequest);
    return this;
  }

  withPublicDataUpdateRequests(publicDataUpdateRequests: PublicDataUpdateRequest[]) {
    this.publicDataUpdateRequests = publicDataUpdateRequests;
    return this;
  }

  pushPublicCall(publicCall: PublicCallRequest) {
    this.publicCallStack.push(publicCall);
    return this;
  }

  withPublicCallStack(publicCallStack: PublicCallRequest[]) {
    this.publicCallStack = publicCallStack;
    return this;
  }

  withGasUsed(gasUsed: Gas) {
    this.gasUsed = gasUsed;
    return this;
  }

  build(): PublicAccumulatedData {
    return new PublicAccumulatedData(
      padArrayEnd(this.noteHashes, NoteHash.empty(), MAX_NOTE_HASHES_PER_TX),
      padArrayEnd(this.nullifiers, Nullifier.empty(), MAX_NULLIFIERS_PER_TX),
      padArrayEnd(this.l2ToL1Msgs, Fr.ZERO, MAX_L2_TO_L1_MSGS_PER_TX),
      padArrayEnd(this.noteEncryptedLogsHashes, LogHash.empty(), MAX_NOTE_ENCRYPTED_LOGS_PER_TX),
      padArrayEnd(this.encryptedLogsHashes, LogHash.empty(), MAX_ENCRYPTED_LOGS_PER_TX),
      padArrayEnd(this.unencryptedLogsHashes, ScopedLogHash.empty(), MAX_UNENCRYPTED_LOGS_PER_TX),
      padArrayEnd(
        this.publicDataUpdateRequests,
        PublicDataUpdateRequest.empty(),
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      ),
      padArrayEnd(this.publicCallStack, PublicCallRequest.empty(), MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX),
      this.gasUsed,
    );
  }

  static fromPublicAccumulatedData(publicAccumulatedData: PublicAccumulatedData): PublicAccumulatedDataBuilder {
    return new PublicAccumulatedDataBuilder()
      .withNoteHashes(publicAccumulatedData.noteHashes)
      .withNullifiers(publicAccumulatedData.nullifiers)
      .withL2ToL1Msgs(publicAccumulatedData.l2ToL1Msgs)
      .withNoteEncryptedLogsHashes(publicAccumulatedData.noteEncryptedLogsHashes)
      .withEncryptedLogsHashes(publicAccumulatedData.encryptedLogsHashes)
      .withUnencryptedLogsHashes(publicAccumulatedData.unencryptedLogsHashes)
      .withPublicDataUpdateRequests(publicAccumulatedData.publicDataUpdateRequests)
      .withPublicCallStack(publicAccumulatedData.publicCallStack)
      .withGasUsed(publicAccumulatedData.gasUsed);
  }
}
