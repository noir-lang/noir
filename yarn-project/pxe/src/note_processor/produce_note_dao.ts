import { type L1NotePayload, type TxHash } from '@aztec/circuit-types';
import { Fr, type PublicKey } from '@aztec/circuits.js';
import { computeNoteHashNonce, siloNullifier } from '@aztec/circuits.js/hash';
import { type Logger } from '@aztec/foundation/log';
import { type AcirSimulator, ContractNotFoundError } from '@aztec/simulator';

import { DeferredNoteDao } from '../database/deferred_note_dao.js';
import { IncomingNoteDao } from '../database/incoming_note_dao.js';
import { OutgoingNoteDao } from '../database/outgoing_note_dao.js';

/**
 * Decodes a note from a transaction that we know was intended for us.
 * Throws if we do not yet have the contract corresponding to the note in our database.
 * Accepts a set of excluded indices, which are indices that have been assigned a note in the same tx.
 * Inserts the index of the note into the excludedIndices set if the note is successfully decoded.
 *
 * @param ivpkM - The public counterpart to the secret key to be used in the decryption of incoming note logs.
 * @param ovpkM - The public counterpart to the secret key to be used in the decryption of outgoing note logs.
 * @param payload - An instance of l1NotePayload.
 * @param txHash - The hash of the transaction that created the note. Equivalent to the first nullifier of the transaction.
 * @param newNoteHashes - New note hashes in this transaction, one of which belongs to this note.
 * @param dataStartIndexForTx - The next available leaf index for the note hash tree for this transaction.
 * @param excludedIndices - Indices that have been assigned a note in the same tx. Notes in a tx can have the same l1NotePayload, we need to find a different index for each replicate.
 * @param simulator - An instance of AcirSimulator.
 * @returns An object containing the incoming note, outgoing note, and deferred note.
 */
export async function produceNoteDaos(
  simulator: AcirSimulator,
  ivpkM: PublicKey | undefined,
  ovpkM: PublicKey | undefined,
  payload: L1NotePayload,
  txHash: TxHash,
  newNoteHashes: Fr[],
  dataStartIndexForTx: number,
  excludedIndices: Set<number>,
  log: Logger,
): Promise<{
  incomingNote: IncomingNoteDao | undefined;
  outgoingNote: OutgoingNoteDao | undefined;
  incomingDeferredNote: DeferredNoteDao | undefined;
}> {
  if (!ivpkM && !ovpkM) {
    throw new Error('Both ivpkM and ovpkM are undefined. Cannot create note.');
  }

  let incomingNote: IncomingNoteDao | undefined;
  let outgoingNote: OutgoingNoteDao | undefined;

  // Note that there are no deferred outgoing notes because we don't need the contract there for anything since we
  // are not attempting to derive a nullifier.
  let incomingDeferredNote: DeferredNoteDao | undefined;

  if (ovpkM) {
    outgoingNote = new OutgoingNoteDao(
      payload.note,
      payload.contractAddress,
      payload.storageSlot,
      payload.noteTypeId,
      txHash,
      ovpkM,
    );
  }

  try {
    if (ivpkM) {
      const { noteHashIndex, nonce, innerNoteHash, siloedNullifier } = await findNoteIndexAndNullifier(
        simulator,
        newNoteHashes,
        txHash,
        payload,
        excludedIndices,
      );
      const index = BigInt(dataStartIndexForTx + noteHashIndex);
      excludedIndices?.add(noteHashIndex);

      incomingNote = new IncomingNoteDao(
        payload.note,
        payload.contractAddress,
        payload.storageSlot,
        payload.noteTypeId,
        txHash,
        nonce,
        innerNoteHash,
        siloedNullifier,
        index,
        ivpkM,
      );
    }
  } catch (e) {
    if (e instanceof ContractNotFoundError) {
      log.warn(e.message);

      if (ivpkM) {
        incomingDeferredNote = new DeferredNoteDao(
          ivpkM,
          payload.note,
          payload.contractAddress,
          payload.storageSlot,
          payload.noteTypeId,
          txHash,
          newNoteHashes,
          dataStartIndexForTx,
        );
      }
    } else {
      log.error(`Could not process note because of "${e}". Discarding note...`);
    }
  }

  return {
    incomingNote,
    outgoingNote,
    incomingDeferredNote,
  };
}

/**
 * Finds nonce, index, inner hash and siloed nullifier for a given note.
 * @dev Finds the index in the note hash tree by computing the note hash with different nonce and see which hash for
 * the current tx matches this value.
 * @remarks This method assists in identifying spent notes in the note hash tree.
 * @param noteHashes - Note hashes in the tx. One of them should correspond to the note we are looking for
 * @param txHash - Hash of a tx the note was emitted in.
 * @param l1NotePayload - The note payload.
 * @param excludedIndices - Indices that have been assigned a note in the same tx. Notes in a tx can have the same
 * l1NotePayload. We need to find a different index for each replicate.
 * @returns Nonce, index, inner hash and siloed nullifier for a given note.
 * @throws If cannot find the nonce for the note.
 */
async function findNoteIndexAndNullifier(
  simulator: AcirSimulator,
  noteHashes: Fr[],
  txHash: TxHash,
  { contractAddress, storageSlot, noteTypeId, note }: L1NotePayload,
  excludedIndices: Set<number>,
) {
  let noteHashIndex = 0;
  let nonce: Fr | undefined;
  let innerNoteHash: Fr | undefined;
  let siloedNoteHash: Fr | undefined;
  let innerNullifier: Fr | undefined;
  const firstNullifier = Fr.fromBuffer(txHash.toBuffer());

  for (; noteHashIndex < noteHashes.length; ++noteHashIndex) {
    if (excludedIndices.has(noteHashIndex)) {
      continue;
    }

    const noteHash = noteHashes[noteHashIndex];
    if (noteHash.equals(Fr.ZERO)) {
      break;
    }

    const expectedNonce = computeNoteHashNonce(firstNullifier, noteHashIndex);
    ({ innerNoteHash, siloedNoteHash, innerNullifier } = await simulator.computeNoteHashAndNullifier(
      contractAddress,
      expectedNonce,
      storageSlot,
      noteTypeId,
      note,
    ));

    if (noteHash.equals(siloedNoteHash)) {
      nonce = expectedNonce;
      break;
    }
  }

  if (!nonce) {
    // NB: this used to warn the user that a decrypted log didn't match any notes.
    // This was previously fine as we didn't chop transient note logs, but now we do (#1641 complete).
    throw new Error('Cannot find a matching note hash for the note.');
  }

  return {
    noteHashIndex,
    nonce,
    innerNoteHash: innerNoteHash!,
    siloedNullifier: siloNullifier(contractAddress, innerNullifier!),
  };
}
