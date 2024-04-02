import { type L1NotePayload, type TxHash } from '@aztec/circuit-types';
import { Fr, type PublicKey } from '@aztec/circuits.js';
import { computeCommitmentNonce, siloNullifier } from '@aztec/circuits.js/hash';
import { type AcirSimulator } from '@aztec/simulator';

import { NoteDao } from '../database/note_dao.js';

/**
 * Decodes a note from a transaction that we know was intended for us.
 * Throws if we do not yet have the contract corresponding to the note in our database.
 * Accepts a set of excluded indices, which are indices that have been assigned a note in the same tx.
 * Inserts the index of the note into the excludedIndices set if the note is successfully decoded.
 *
 * @param publicKey - The public counterpart to the private key to be used in note decryption.
 * @param payload - An instance of l1NotePayload.
 * @param txHash - The hash of the transaction that created the note. Equivalent to the first nullifier of the transaction.
 * @param newNoteHashes - New note hashes in this transaction, one of which belongs to this note.
 * @param dataStartIndexForTx - The next available leaf index for the note hash tree for this transaction.
 * @param excludedIndices - Indices that have been assigned a note in the same tx. Notes in a tx can have the same l1NotePayload, we need to find a different index for each replicate.
 * @param simulator - An instance of AcirSimulator.
 * @returns an instance of NoteDao, or throws. inserts the index of the note into the excludedIndices set.
 */
export async function produceNoteDao(
  simulator: AcirSimulator,
  publicKey: PublicKey,
  payload: L1NotePayload,
  txHash: TxHash,
  newNoteHashes: Fr[],
  dataStartIndexForTx: number,
  excludedIndices: Set<number>,
): Promise<NoteDao> {
  const { commitmentIndex, nonce, innerNoteHash, siloedNullifier } = await findNoteIndexAndNullifier(
    simulator,
    newNoteHashes,
    txHash,
    payload,
    excludedIndices,
  );
  const index = BigInt(dataStartIndexForTx + commitmentIndex);
  excludedIndices?.add(commitmentIndex);
  return new NoteDao(
    payload.note,
    payload.contractAddress,
    payload.storageSlot,
    payload.noteTypeId,
    txHash,
    nonce,
    innerNoteHash,
    siloedNullifier,
    index,
    publicKey,
  );
}

/**
 * Find the index of the note in the note hash tree by computing the note hash with different nonce and see which
 * commitment for the current tx matches this value.
 * Compute a nullifier for a given l1NotePayload.
 * The nullifier is calculated using the private key of the account,
 * contract address, and the note associated with the l1NotePayload.
 * This method assists in identifying spent commitments in the private state.
 * @param commitments - Commitments in the tx. One of them should be the note's commitment.
 * @param txHash - First nullifier in the tx.
 * @param l1NotePayload - An instance of l1NotePayload.
 * @param excludedIndices - Indices that have been assigned a note in the same tx. Notes in a tx can have the same
 * l1NotePayload. We need to find a different index for each replicate.
 * @returns Information for a decrypted note, including the index of its commitment, nonce, inner note
 * hash, and the siloed nullifier. Throw if cannot find the nonce for the note.
 */
async function findNoteIndexAndNullifier(
  simulator: AcirSimulator,
  commitments: Fr[],
  txHash: TxHash,
  { contractAddress, storageSlot, noteTypeId, note }: L1NotePayload,
  excludedIndices: Set<number>,
) {
  let commitmentIndex = 0;
  let nonce: Fr | undefined;
  let innerNoteHash: Fr | undefined;
  let siloedNoteHash: Fr | undefined;
  let uniqueSiloedNoteHash: Fr | undefined;
  let innerNullifier: Fr | undefined;
  const firstNullifier = Fr.fromBuffer(txHash.toBuffer());

  for (; commitmentIndex < commitments.length; ++commitmentIndex) {
    if (excludedIndices.has(commitmentIndex)) {
      continue;
    }

    const commitment = commitments[commitmentIndex];
    if (commitment.equals(Fr.ZERO)) {
      break;
    }

    const expectedNonce = computeCommitmentNonce(firstNullifier, commitmentIndex);
    ({ innerNoteHash, siloedNoteHash, uniqueSiloedNoteHash, innerNullifier } =
      await simulator.computeNoteHashAndNullifier(contractAddress, expectedNonce, storageSlot, noteTypeId, note));
    if (commitment.equals(uniqueSiloedNoteHash)) {
      nonce = expectedNonce;
      break;
    }
  }

  if (!nonce) {
    let errorString;
    if (siloedNoteHash == undefined) {
      errorString = 'Cannot find a matching commitment for the note.';
    } else {
      errorString = `We decrypted a log, but couldn't find a corresponding note in the tree.
This might be because the note was nullified in the same tx which created it.
In that case, everything is fine. To check whether this is the case, look back through
the logs for a notification
'important: chopped commitment for siloed inner hash note
${siloedNoteHash.toString()}'.
If you can see that notification. Everything's fine.
If that's not the case, and you can't find such a notification, something has gone wrong.
There could be a problem with the way you've defined a custom note, or with the way you're
serializing / deserializing / hashing / encrypting / decrypting that note.
Please see the following github issue to track an improvement that we're working on:
https://github.com/AztecProtocol/aztec-packages/issues/1641`;
    }

    throw new Error(errorString);
  }

  return {
    commitmentIndex,
    nonce,
    innerNoteHash: innerNoteHash!,
    siloedNullifier: siloNullifier(contractAddress, innerNullifier!),
  };
}
