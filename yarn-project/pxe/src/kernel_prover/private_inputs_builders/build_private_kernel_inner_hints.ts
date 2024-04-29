import {
  type MAX_NEW_NOTE_HASHES_PER_CALL,
  type PrivateCircuitPublicInputs,
  PrivateKernelInnerHints,
} from '@aztec/circuits.js';
import { type Tuple } from '@aztec/foundation/serialize';

export function buildPrivateKernelInnerHints(
  publicInputs: PrivateCircuitPublicInputs,
  noteHashNullifierCounterMap: Map<number, number>,
) {
  const nullifierCounters = publicInputs.newNoteHashes.map(
    n => noteHashNullifierCounterMap.get(n.counter) ?? 0,
  ) as Tuple<number, typeof MAX_NEW_NOTE_HASHES_PER_CALL>;

  return new PrivateKernelInnerHints(nullifierCounters);
}
