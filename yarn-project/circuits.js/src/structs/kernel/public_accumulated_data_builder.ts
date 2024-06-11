import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';

import {
  MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_ENCRYPTED_LOGS_PER_TX,
  MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  MAX_UNENCRYPTED_LOGS_PER_TX,
} from '../../constants.gen.js';
import { CallRequest } from '../call_request.js';
import { Gas } from '../gas.js';
import { LogHash } from '../log_hash.js';
import { NoteHash } from '../note_hash.js';
import { Nullifier } from '../nullifier.js';
import { PublicDataUpdateRequest } from '../public_data_update_request.js';
import { PublicAccumulatedData } from './public_accumulated_data.js';

/**
 * TESTS-ONLY CLASS
 * Builder for PublicAccumulatedData, used to conveniently construct instances for testing,
 * as PublicAccumulatedData is (or will shortly be) immutable.
 *
 */
export class PublicAccumulatedDataBuilder {
  private newNoteHashes: NoteHash[] = [];
  private newNullifiers: Nullifier[] = [];
  private newL2ToL1Msgs: Fr[] = [];
  private noteEncryptedLogsHashes: LogHash[] = [];
  private encryptedLogsHashes: LogHash[] = [];
  private unencryptedLogsHashes: LogHash[] = [];
  private publicDataUpdateRequests: PublicDataUpdateRequest[] = [];
  private publicCallStack: CallRequest[] = [];
  private gasUsed: Gas = Gas.empty();

  pushNewNoteHash(newNoteHash: NoteHash) {
    this.newNoteHashes.push(newNoteHash);
    return this;
  }

  withNewNoteHashes(newNoteHashes: NoteHash[]) {
    this.newNoteHashes = newNoteHashes;
    return this;
  }

  pushNewNullifier(newNullifier: Nullifier) {
    this.newNullifiers.push(newNullifier);
    return this;
  }

  withNewNullifiers(newNullifiers: Nullifier[]) {
    this.newNullifiers = newNullifiers;
    return this;
  }

  pushNewL2ToL1Msg(newL2ToL1Msg: Fr) {
    this.newL2ToL1Msgs.push(newL2ToL1Msg);
    return this;
  }

  withNewL2ToL1Msgs(newL2ToL1Msgs: Fr[]) {
    this.newL2ToL1Msgs = newL2ToL1Msgs;
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

  pushUnencryptedLogsHash(unencryptedLogsHash: LogHash) {
    this.unencryptedLogsHashes.push(unencryptedLogsHash);
    return this;
  }

  withUnencryptedLogsHashes(unencryptedLogsHashes: LogHash[]) {
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

  pushPublicCall(publicCall: CallRequest) {
    this.publicCallStack.push(publicCall);
    return this;
  }

  withPublicCallStack(publicCallStack: CallRequest[]) {
    this.publicCallStack = publicCallStack;
    return this;
  }

  withGasUsed(gasUsed: Gas) {
    this.gasUsed = gasUsed;
    return this;
  }

  build(): PublicAccumulatedData {
    return new PublicAccumulatedData(
      padArrayEnd(this.newNoteHashes, NoteHash.empty(), MAX_NEW_NOTE_HASHES_PER_TX),
      padArrayEnd(this.newNullifiers, Nullifier.empty(), MAX_NEW_NULLIFIERS_PER_TX),
      padArrayEnd(this.newL2ToL1Msgs, Fr.ZERO, MAX_NEW_L2_TO_L1_MSGS_PER_TX),
      padArrayEnd(this.noteEncryptedLogsHashes, LogHash.empty(), MAX_NOTE_ENCRYPTED_LOGS_PER_TX),
      padArrayEnd(this.encryptedLogsHashes, LogHash.empty(), MAX_ENCRYPTED_LOGS_PER_TX),
      padArrayEnd(this.unencryptedLogsHashes, LogHash.empty(), MAX_UNENCRYPTED_LOGS_PER_TX),
      padArrayEnd(
        this.publicDataUpdateRequests,
        PublicDataUpdateRequest.empty(),
        MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
      ),
      padArrayEnd(this.publicCallStack, CallRequest.empty(), MAX_PUBLIC_CALL_STACK_LENGTH_PER_TX),
      this.gasUsed,
    );
  }

  static fromPublicAccumulatedData(publicAccumulatedData: PublicAccumulatedData): PublicAccumulatedDataBuilder {
    return new PublicAccumulatedDataBuilder()
      .withNewNoteHashes(publicAccumulatedData.newNoteHashes)
      .withNewNullifiers(publicAccumulatedData.newNullifiers)
      .withNewL2ToL1Msgs(publicAccumulatedData.newL2ToL1Msgs)
      .withNoteEncryptedLogsHashes(publicAccumulatedData.noteEncryptedLogsHashes)
      .withEncryptedLogsHashes(publicAccumulatedData.encryptedLogsHashes)
      .withUnencryptedLogsHashes(publicAccumulatedData.unencryptedLogsHashes)
      .withPublicDataUpdateRequests(publicAccumulatedData.publicDataUpdateRequests)
      .withPublicCallStack(publicAccumulatedData.publicCallStack)
      .withGasUsed(publicAccumulatedData.gasUsed);
  }
}
