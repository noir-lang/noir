import {
  type Fr,
  KeyValidationHint,
  MAX_KEY_VALIDATION_REQUESTS_PER_TX,
  MAX_NOTE_HASHES_PER_TX,
  MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
  MAX_NULLIFIERS_PER_TX,
  MAX_NULLIFIER_READ_REQUESTS_PER_TX,
  MembershipWitness,
  NULLIFIER_TREE_HEIGHT,
  PRIVATE_RESET_VARIANTS,
  type PrivateKernelData,
  PrivateKernelResetCircuitPrivateInputs,
  type PrivateKernelResetCircuitPrivateInputsVariants,
  PrivateKernelResetHints,
  type ReadRequest,
  type ScopedKeyValidationRequestAndGenerator,
  ScopedNoteHash,
  ScopedNullifier,
  ScopedReadRequest,
  buildNoteHashReadRequestHints,
  buildNullifierReadRequestHints,
  buildTransientDataHints,
  getNonEmptyItems,
} from '@aztec/circuits.js';
import { makeTuple } from '@aztec/foundation/array';
import { type Tuple } from '@aztec/foundation/serialize';
import type { ExecutionResult } from '@aztec/simulator';

import { type ProvingDataOracle } from '../proving_data_oracle.js';

function getNullifierReadRequestHints<PENDING extends number, SETTLED extends number>(
  nullifierReadRequests: Tuple<ScopedReadRequest, typeof MAX_NULLIFIER_READ_REQUESTS_PER_TX>,
  nullifiers: Tuple<ScopedNullifier, typeof MAX_NULLIFIERS_PER_TX>,
  oracle: ProvingDataOracle,
  sizePending: PENDING,
  sizeSettled: SETTLED,
  futureNullifiers: ScopedNullifier[],
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

  return buildNullifierReadRequestHints(
    { getNullifierMembershipWitness },
    nullifierReadRequests,
    nullifiers,
    sizePending,
    sizeSettled,
    futureNullifiers,
  );
}

async function getMasterSecretKeysAndAppKeyGenerators(
  keyValidationRequests: Tuple<ScopedKeyValidationRequestAndGenerator, typeof MAX_KEY_VALIDATION_REQUESTS_PER_TX>,
  oracle: ProvingDataOracle,
) {
  const keysHints = makeTuple(MAX_KEY_VALIDATION_REQUESTS_PER_TX, KeyValidationHint.empty);

  let keyIndex = 0;
  for (let i = 0; i < keyValidationRequests.length; ++i) {
    const request = keyValidationRequests[i].request;
    if (request.isEmpty()) {
      break;
    }
    const secretKeys = await oracle.getMasterSecretKey(request.request.pkM);
    keysHints[keyIndex] = new KeyValidationHint(secretKeys, i);
    keyIndex++;
  }
  return {
    keysCount: keyIndex,
    keysHints,
  };
}

export async function buildPrivateKernelResetInputs(
  executionStack: ExecutionResult[],
  previousKernelData: PrivateKernelData,
  noteHashLeafIndexMap: Map<bigint, bigint>,
  noteHashNullifierCounterMap: Map<number, number>,
  oracle: ProvingDataOracle,
) {
  const publicInputs = previousKernelData.publicInputs;
  // Use max sizes, they will be trimmed down later.

  const futureNoteHashes = collectNested(executionStack, executionResult => {
    const nonEmptyNoteHashes = getNonEmptyItems(executionResult.callStackItem.publicInputs.noteHashes);
    return nonEmptyNoteHashes.map(
      noteHash =>
        new ScopedNoteHash(noteHash, executionResult.callStackItem.publicInputs.callContext.storageContractAddress),
    );
  });

  const {
    numPendingReadHints: noteHashPendingReadHints,
    numSettledReadHints: noteHashSettledReadHints,
    hints: noteHashReadRequestHints,
  } = await buildNoteHashReadRequestHints(
    oracle,
    publicInputs.validationRequests.noteHashReadRequests,
    publicInputs.end.noteHashes,
    noteHashLeafIndexMap,
    MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    MAX_NOTE_HASH_READ_REQUESTS_PER_TX,
    futureNoteHashes,
  );

  const futureNullifiers = collectNested(executionStack, executionResult => {
    const nonEmptyNullifiers = getNonEmptyItems(executionResult.callStackItem.publicInputs.nullifiers);
    return nonEmptyNullifiers.map(
      nullifier =>
        new ScopedNullifier(nullifier, executionResult.callStackItem.publicInputs.callContext.storageContractAddress),
    );
  });

  const {
    numPendingReadHints: nullifierPendingReadHints,
    numSettledReadHints: nullifierSettledReadHints,
    hints: nullifierReadRequestHints,
  } = await getNullifierReadRequestHints(
    publicInputs.validationRequests.nullifierReadRequests,
    publicInputs.end.nullifiers,
    oracle,
    MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    MAX_NULLIFIER_READ_REQUESTS_PER_TX,
    futureNullifiers,
  );

  const { keysCount, keysHints } = await getMasterSecretKeysAndAppKeyGenerators(
    publicInputs.validationRequests.scopedKeyValidationRequestsAndGenerators,
    oracle,
  );

  const futureNoteHashReads = collectNestedReadRequests(
    executionStack,
    executionResult => executionResult.callStackItem.publicInputs.noteHashReadRequests,
  );
  const futureNullifierReads = collectNestedReadRequests(
    executionStack,
    executionResult => executionResult.callStackItem.publicInputs.nullifierReadRequests,
  );

  const [transientNullifierIndexesForNoteHashes, transientNoteHashIndexesForNullifiers] = buildTransientDataHints(
    publicInputs.end.noteHashes,
    publicInputs.end.nullifiers,
    futureNoteHashReads,
    futureNullifierReads,
    noteHashNullifierCounterMap,
    MAX_NOTE_HASHES_PER_TX,
    MAX_NULLIFIERS_PER_TX,
  );

  let privateInputs;

  for (const [sizeTag, hintSizes] of Object.entries(PRIVATE_RESET_VARIANTS)) {
    if (
      hintSizes.NOTE_HASH_PENDING_AMOUNT >= noteHashPendingReadHints &&
      hintSizes.NOTE_HASH_SETTLED_AMOUNT >= noteHashSettledReadHints &&
      hintSizes.NULLIFIER_PENDING_AMOUNT >= nullifierPendingReadHints &&
      hintSizes.NULLIFIER_SETTLED_AMOUNT >= nullifierSettledReadHints &&
      hintSizes.NULLIFIER_KEYS >= keysCount
    ) {
      privateInputs = new PrivateKernelResetCircuitPrivateInputs(
        previousKernelData,
        new PrivateKernelResetHints(
          transientNullifierIndexesForNoteHashes,
          transientNoteHashIndexesForNullifiers,
          noteHashReadRequestHints,
          nullifierReadRequestHints,
          keysHints,
        ).trimToSizes(
          hintSizes.NOTE_HASH_PENDING_AMOUNT,
          hintSizes.NOTE_HASH_SETTLED_AMOUNT,
          hintSizes.NULLIFIER_PENDING_AMOUNT,
          hintSizes.NULLIFIER_SETTLED_AMOUNT,
          hintSizes.NULLIFIER_KEYS,
        ),
        sizeTag,
      );
      break;
    }
  }

  if (!privateInputs) {
    throw new Error('No private inputs found for the given hint sizes.');
  }

  return privateInputs as PrivateKernelResetCircuitPrivateInputsVariants;
}

function collectNested<T>(
  executionStack: ExecutionResult[],
  extractExecutionItems: (execution: ExecutionResult) => T[],
): T[] {
  const thisExecutionReads = executionStack.flatMap(extractExecutionItems);

  return thisExecutionReads.concat(
    executionStack.flatMap(({ nestedExecutions }) => collectNested(nestedExecutions, extractExecutionItems)),
  );
}

function collectNestedReadRequests(
  executionStack: ExecutionResult[],
  extractReadRequests: (execution: ExecutionResult) => ReadRequest[],
): ScopedReadRequest[] {
  return collectNested(executionStack, executionResult => {
    const nonEmptyReadRequests = getNonEmptyItems(extractReadRequests(executionResult));
    return nonEmptyReadRequests.map(
      readRequest =>
        new ScopedReadRequest(
          readRequest,
          executionResult.callStackItem.publicInputs.callContext.storageContractAddress,
        ),
    );
  });
}
