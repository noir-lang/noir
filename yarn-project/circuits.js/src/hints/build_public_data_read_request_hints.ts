import { type Tuple } from '@aztec/foundation/serialize';

import {
  type MAX_PUBLIC_DATA_HINTS,
  type MAX_PUBLIC_DATA_READS_PER_TX,
  type MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
} from '../constants.gen.js';
import {
  type PublicDataRead,
  PublicDataReadRequestHintsBuilder,
  type PublicDataUpdateRequest,
} from '../structs/index.js';
import { type PublicDataHint } from '../structs/public_data_hint.js';
import { countAccumulatedItems } from '../utils/index.js';

export function buildPublicDataReadRequestHints(
  publicDataReads: Tuple<PublicDataRead, typeof MAX_PUBLIC_DATA_READS_PER_TX>,
  publicDataUpdateRequests: Tuple<PublicDataUpdateRequest, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
  publicDataHints: Tuple<PublicDataHint, typeof MAX_PUBLIC_DATA_HINTS>,
) {
  const builder = new PublicDataReadRequestHintsBuilder();

  const numReadRequests = countAccumulatedItems(publicDataReads);
  for (let i = 0; i < numReadRequests; ++i) {
    const rr = publicDataReads[i];
    // TODO: Add counters to reads and writes.
    const writeIndex = publicDataUpdateRequests.findIndex(
      w => w.leafSlot.equals(rr.leafSlot) && w.newValue.equals(rr.value),
    );
    if (writeIndex !== -1) {
      builder.addPendingReadRequest(i, writeIndex);
    } else {
      const hintIndex = publicDataHints.findIndex(h => h.leafSlot.equals(rr.leafSlot));
      if (hintIndex === -1) {
        throw new Error('Cannot find a pending write or a data hint for the read request.');
      }
      if (!publicDataHints[hintIndex].value.equals(rr.value)) {
        throw new Error('Value being read does not match existing public data or pending writes.');
      }
      builder.addLeafDataReadRequest(i, hintIndex);
    }
  }

  return builder.toHints();
}
