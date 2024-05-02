import {
  type Fr,
  GrumpkinScalar,
  type MAX_ENCRYPTED_LOGS_PER_TX,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
  type MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  type MAX_UNENCRYPTED_LOGS_PER_TX,
  MembershipWitness,
  NULLIFIER_TREE_HEIGHT,
  type Nullifier,
  type NullifierKeyValidationRequestContext,
  type PrivateKernelCircuitPublicInputs,
  PrivateKernelTailHints,
  type ReadRequestContext,
  type SideEffect,
  type SideEffectType,
  buildNoteHashReadRequestHints,
  buildNullifierReadRequestHints,
  buildTransientDataHints,
  sortByCounterGetSortedHints,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { type Tuple } from '@aztec/foundation/serialize';

import { type ProvingDataOracle } from '../proving_data_oracle.js';

/** @deprecated Use sortByCounterGetSortedHints instead */
function sortSideEffects<T extends SideEffectType, K extends number>(
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

function getNullifierReadRequestHints(
  nullifierReadRequests: Tuple<ReadRequestContext, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
  nullifiers: Tuple<Nullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>,
  oracle: ProvingDataOracle,
) {
  const getNullifierMembershipWitness = async (nullifier: Fr) => {
    const res = await oracle.getNullifierMembershipWitness(nullifier);
    if (!res) {
      throw new Error(`Cannot find the leaf for nullifier ${nullifier.toBigInt()}.`);
    }

    const { index, siblingPath, leafPreimage } = res;
    return {
      membershipWitness: new MembershipWitness(
        NULLIFIER_TREE_HEIGHT,
        index,
        siblingPath.toTuple<typeof NULLIFIER_TREE_HEIGHT>(),
      ),
      leafPreimage,
    };
  };

  return buildNullifierReadRequestHints({ getNullifierMembershipWitness }, nullifierReadRequests, nullifiers);
}

async function getMasterNullifierSecretKeys(
  nullifierKeyValidationRequests: Tuple<
    NullifierKeyValidationRequestContext,
    typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX
  >,
  oracle: ProvingDataOracle,
) {
  const keys = makeTuple(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, GrumpkinScalar.zero);
  for (let i = 0; i < nullifierKeyValidationRequests.length; ++i) {
    const request = nullifierKeyValidationRequests[i];
    if (request.isEmpty()) {
      break;
    }
    keys[i] = await oracle.getMasterNullifierSecretKey(request.masterNullifierPublicKey);
  }
  return keys;
}

export async function buildPrivateKernelTailHints(
  publicInputs: PrivateKernelCircuitPublicInputs,
  noteHashLeafIndexMap: Map<bigint, bigint>,
  oracle: ProvingDataOracle,
) {
  const noteHashReadRequestHints = await buildNoteHashReadRequestHints(
    oracle,
    publicInputs.validationRequests.noteHashReadRequests,
    publicInputs.end.newNoteHashes,
    noteHashLeafIndexMap,
  );

  const nullifierReadRequestHints = await getNullifierReadRequestHints(
    publicInputs.validationRequests.nullifierReadRequests,
    publicInputs.end.newNullifiers,
    oracle,
  );

  const masterNullifierSecretKeys = await getMasterNullifierSecretKeys(
    publicInputs.validationRequests.nullifierKeyValidationRequests,
    oracle,
  );

  const [sortedNoteHashes, sortedNoteHashesIndexes] = sortByCounterGetSortedHints(
    publicInputs.end.newNoteHashes,
    MAX_NEW_NOTE_HASHES_PER_TX,
  );

  const [sortedNullifiers, sortedNullifiersIndexes] = sortByCounterGetSortedHints(
    publicInputs.end.newNullifiers,
    MAX_NEW_NULLIFIERS_PER_TX,
  );

  const [sortedEncryptedLogHashes, sortedEncryptedLogHashesIndexes] = sortSideEffects<
    SideEffect,
    typeof MAX_ENCRYPTED_LOGS_PER_TX
  >(publicInputs.end.encryptedLogsHashes);

  const [sortedUnencryptedLogHashes, sortedUnencryptedLogHashesIndexes] = sortSideEffects<
    SideEffect,
    typeof MAX_UNENCRYPTED_LOGS_PER_TX
  >(publicInputs.end.unencryptedLogsHashes);

  const [transientNullifierIndexesForNoteHashes, transientNoteHashIndexesForNullifiers] = buildTransientDataHints(
    sortedNoteHashes,
    sortedNullifiers,
    MAX_NEW_NOTE_HASHES_PER_TX,
    MAX_NEW_NULLIFIERS_PER_TX,
  );

  return new PrivateKernelTailHints(
    transientNullifierIndexesForNoteHashes,
    transientNoteHashIndexesForNullifiers,
    noteHashReadRequestHints,
    nullifierReadRequestHints,
    masterNullifierSecretKeys,
    sortedNoteHashes,
    sortedNoteHashesIndexes,
    sortedNullifiers,
    sortedNullifiersIndexes,
    sortedEncryptedLogHashes,
    sortedEncryptedLogHashesIndexes,
    sortedUnencryptedLogHashes,
    sortedUnencryptedLogHashesIndexes,
  );
}
