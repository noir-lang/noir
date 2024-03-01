import {
  Fr,
  GrumpkinScalar,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MembershipWitness,
  NULLIFIER_TREE_HEIGHT,
  NullifierKeyValidationRequestContext,
  NullifierReadRequestResetHintsBuilder,
  ReadRequestContext,
  SideEffect,
  SideEffectLinkedToNoteHash,
  SideEffectType,
} from '@aztec/circuits.js';
import { siloNullifier } from '@aztec/circuits.js/hash';
import { makeTuple } from '@aztec/foundation/array';
import { Tuple } from '@aztec/foundation/serialize';

import { ProvingDataOracle } from './proving_data_oracle.js';

export class HintsBuilder {
  constructor(private oracle: ProvingDataOracle) {}

  sortSideEffects<T extends SideEffectType, K extends number>(
    sideEffects: Tuple<T, K>,
  ): [Tuple<T, K>, Tuple<number, K>] {
    const sorted = sideEffects
      .map((sideEffect, index) => ({ sideEffect, index }))
      .sort((a, b) => {
        // Empty ones go to the right
        if (a.sideEffect.isEmpty()) {
          return 1;
        }
        return Number(a.sideEffect.counter.toBigInt() - b.sideEffect.counter.toBigInt());
      });

    const originalToSorted = sorted.map(() => 0);
    sorted.forEach(({ index }, i) => {
      originalToSorted[index] = i;
    });

    return [sorted.map(({ sideEffect }) => sideEffect) as Tuple<T, K>, originalToSorted as Tuple<number, K>];
  }

  /**
   * Performs the matching between an array of read request and an array of note hashes. This produces
   * hints for the private kernel tail circuit to efficiently match a read request with the corresponding
   * note hash. Several read requests might be pointing to the same note hash. It is therefore valid
   * to return more than one hint with the same index (contrary to getNullifierHints).
   *
   * @param noteHashReadRequests - The array of read requests.
   * @param noteHashes - The array of note hashes.
   * @returns An array of hints where each element is the index of the note hash in note hashes array
   *  corresponding to the read request. In other words we have readRequests[i] == noteHashes[hints[i]].
   */
  getNoteHashReadRequestHints(
    noteHashReadRequests: Tuple<SideEffect, typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX>,
    noteHashes: Tuple<SideEffect, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
  ): Tuple<Fr, typeof MAX_NOTE_HASH_READ_REQUESTS_PER_TX> {
    const hints = makeTuple(MAX_NOTE_HASH_READ_REQUESTS_PER_TX, Fr.zero);
    for (let i = 0; i < MAX_NOTE_HASH_READ_REQUESTS_PER_TX && !noteHashReadRequests[i].isEmpty(); i++) {
      const equalToRR = (cmt: SideEffect) => cmt.value.equals(noteHashReadRequests[i].value);
      const result = noteHashes.findIndex(equalToRR);
      if (result == -1) {
        throw new Error(
          `The read request at index ${i} ${noteHashReadRequests[i].toString()} does not match to any note hash.`,
        );
      } else {
        hints[i] = new Fr(result);
      }
    }
    return hints;
  }

  async getNullifierReadRequestResetHints(
    nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
    nullifiers: Tuple<SideEffectLinkedToNoteHash, typeof MAX_NEW_NULLIFIERS_PER_TX>,
  ) {
    // TODO - Should be comparing un-siloed values and contract addresses.
    const builder = new NullifierReadRequestResetHintsBuilder();
    const nullifierIndexMap: Map<bigint, number> = new Map();
    nullifiers.forEach((n, i) => nullifierIndexMap.set(n.value.toBigInt(), i));
    const siloedReadRequestValues = nullifierReadRequests.map(r =>
      r.isEmpty() ? Fr.ZERO : siloNullifier(r.contractAddress, r.value),
    );
    for (let i = 0; i < nullifierReadRequests.length; ++i) {
      const value = siloedReadRequestValues[i];
      if (value.isZero()) {
        break;
      }
      const pendingValueIndex = nullifierIndexMap.get(value.toBigInt());
      if (pendingValueIndex !== undefined) {
        builder.addPendingReadRequest(i, pendingValueIndex);
      } else {
        const membershipWitness = await this.oracle.getNullifierMembershipWitness(0, value);
        if (!membershipWitness) {
          throw new Error('Read request is reading an unknown nullifier value.');
        }
        builder.addSettledReadRequest(
          i,
          new MembershipWitness(
            NULLIFIER_TREE_HEIGHT,
            membershipWitness.index,
            membershipWitness.siblingPath.toTuple<typeof NULLIFIER_TREE_HEIGHT>(),
          ),
          membershipWitness.leafPreimage,
        );
      }
    }
    return builder.toHints();
  }

  /**
   * Performs the matching between an array of nullified note hashes and an array of note hashes. This produces
   * hints for the private kernel tail circuit to efficiently match a nullifier with the corresponding
   * note hash. Note that the same note hash value might appear more than once in the note hashes
   * (resp. nullified note hashes) array. It is crucial in this case that each hint points to a different index
   * of the nullified note hashes array. Otherwise, the private kernel will fail to validate.
   *
   * @param nullifiedNoteHashes - The array of nullified note hashes.
   * @param noteHashes - The array of note hashes.
   * @returns An array of hints where each element is the index of the note hash in note hashes array
   *  corresponding to the nullified note hash. In other words we have nullifiedNoteHashes[i] == noteHashes[hints[i]].
   */
  getNullifierHints(
    nullifiedNoteHashes: Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX>,
    noteHashes: Tuple<SideEffect, typeof MAX_NEW_NOTE_HASHES_PER_TX>,
  ): Tuple<Fr, typeof MAX_NEW_NULLIFIERS_PER_TX> {
    const hints = makeTuple(MAX_NEW_NULLIFIERS_PER_TX, Fr.zero);
    const alreadyUsed = new Set<number>();
    for (let i = 0; i < MAX_NEW_NULLIFIERS_PER_TX; i++) {
      if (!nullifiedNoteHashes[i].isZero()) {
        const result = noteHashes.findIndex(
          (cmt: SideEffect, index: number) => cmt.value.equals(nullifiedNoteHashes[i]) && !alreadyUsed.has(index),
        );
        alreadyUsed.add(result);
        if (result == -1) {
          throw new Error(
            `The nullified note hash at index ${i} with value ${nullifiedNoteHashes[
              i
            ].toString()} does not match to any note hash.`,
          );
        } else {
          hints[i] = new Fr(result);
        }
      }
    }
    return hints;
  }

  async getMasterNullifierSecretKeys(
    nullifierKeyValidationRequests: Tuple<
      NullifierKeyValidationRequestContext,
      typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX
    >,
  ) {
    const keys = makeTuple(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, GrumpkinScalar.zero);
    for (let i = 0; i < nullifierKeyValidationRequests.length; ++i) {
      const request = nullifierKeyValidationRequests[i];
      if (request.isEmpty()) {
        break;
      }
      keys[i] = await this.oracle.getMasterNullifierSecretKey(request.publicKey);
    }
    return keys;
  }
}
