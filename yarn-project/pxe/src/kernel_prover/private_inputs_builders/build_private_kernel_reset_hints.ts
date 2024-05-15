import {
  type Fr,
  GrumpkinScalar,
  MAX_NEW_NOTE_HASHES_PER_TX,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX,
  type MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MembershipWitness,
  NULLIFIER_TREE_HEIGHT,
  type PrivateKernelCircuitPublicInputs,
  PrivateKernelResetHints,
  type ScopedNullifier,
  type ScopedNullifierKeyValidationRequest,
  type ScopedReadRequest,
  buildNoteHashReadRequestHints,
  buildNullifierReadRequestHints,
  buildTransientDataHints,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { type Tuple } from '@aztec/foundation/serialize';

import { type ProvingDataOracle } from '../proving_data_oracle.js';

function getNullifierReadRequestHints(
  nullifierReadRequests: Tuple<ScopedReadRequest, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
  nullifiers: Tuple<ScopedNullifier, typeof MAX_NEW_NULLIFIERS_PER_TX>,
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
    ScopedNullifierKeyValidationRequest,
    typeof MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX
  >,
  oracle: ProvingDataOracle,
) {
  const keys = makeTuple(MAX_NULLIFIER_KEY_VALIDATION_REQUESTS_PER_TX, GrumpkinScalar.zero);
  for (let i = 0; i < nullifierKeyValidationRequests.length; ++i) {
    const request = nullifierKeyValidationRequests[i].request;
    if (request.isEmpty()) {
      break;
    }
    keys[i] = await oracle.getMasterNullifierSecretKey(request.masterNullifierPublicKey);
  }
  return keys;
}

export async function buildPrivateKernelResetHints(
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

  const [transientNullifierIndexesForNoteHashes, transientNoteHashIndexesForNullifiers] = buildTransientDataHints(
    publicInputs.end.newNoteHashes,
    publicInputs.end.newNullifiers,
    MAX_NEW_NOTE_HASHES_PER_TX,
    MAX_NEW_NULLIFIERS_PER_TX,
  );

  return new PrivateKernelResetHints(
    transientNullifierIndexesForNoteHashes,
    transientNoteHashIndexesForNullifiers,
    noteHashReadRequestHints,
    nullifierReadRequestHints,
    masterNullifierSecretKeys,
  );
}
