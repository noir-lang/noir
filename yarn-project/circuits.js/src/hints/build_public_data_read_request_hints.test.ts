import { makeTuple } from '@aztec/foundation/array';
import { padArrayEnd } from '@aztec/foundation/collection';
import { Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';

import {
  MAX_PUBLIC_DATA_HINTS,
  MAX_PUBLIC_DATA_READS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
} from '../constants.gen.js';
import {
  LeafDataReadHint,
  PendingReadHint,
  PublicDataHint,
  PublicDataRead,
  PublicDataUpdateRequest,
  ReadRequestStatus,
} from '../structs/index.js';
import { buildPublicDataReadRequestHints } from './build_public_data_read_request_hints.js';

describe('buildPublicDataReadRequestHints', () => {
  let publicDataReads: Tuple<PublicDataRead, typeof MAX_PUBLIC_DATA_READS_PER_TX>;
  let expectedStatuses: Tuple<ReadRequestStatus, typeof MAX_PUBLIC_DATA_READS_PER_TX>;
  let expectedPendingHints: Tuple<PendingReadHint, typeof MAX_PUBLIC_DATA_READS_PER_TX>;
  let expectedLeafDataHints: Tuple<LeafDataReadHint, typeof MAX_PUBLIC_DATA_READS_PER_TX>;

  const makePublicDataWrite = (leafSlot: number, value: number) =>
    new PublicDataUpdateRequest(new Fr(leafSlot), new Fr(value));
  const makePublicDataHint = (slot: number, value: number) => {
    const hint = PublicDataHint.empty();
    hint.leafSlot = new Fr(slot);
    hint.value = new Fr(value);
    return hint;
  };
  const makePublicDataRead = (leafSlot: number, value: number) => new PublicDataRead(new Fr(leafSlot), new Fr(value));
  const makePendingHint = (readRequestIndex: number, hintIndex: number) =>
    new PendingReadHint(readRequestIndex, hintIndex);
  const makeLeafDataHint = (readRequestIndex: number, hintIndex: number) =>
    new LeafDataReadHint(readRequestIndex, hintIndex);

  const publicDataUpdateRequests = padArrayEnd(
    [makePublicDataWrite(55, 5555), makePublicDataWrite(77, 7777), makePublicDataWrite(99, 9999)],
    PublicDataUpdateRequest.empty(),
    MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  );

  const publicDataHints = padArrayEnd(
    [
      makePublicDataHint(11, 100),
      makePublicDataHint(22, 200),
      makePublicDataHint(33, 300),
      makePublicDataHint(55, 500),
      makePublicDataHint(77, 0),
      makePublicDataHint(99, 900),
    ],
    PublicDataHint.empty(),
    MAX_PUBLIC_DATA_HINTS,
  );

  const buildHints = () => buildPublicDataReadRequestHints(publicDataReads, publicDataUpdateRequests, publicDataHints);

  const buildAndCheckHints = () => {
    const hints = buildHints();
    expect(hints.readRequestStatuses).toEqual(expectedStatuses);
    expect(hints.pendingReadHints).toEqual(expectedPendingHints);
    expect(hints.leafDataReadHints).toEqual(expectedLeafDataHints);
  };

  beforeEach(() => {
    publicDataReads = makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty);
    expectedStatuses = makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, ReadRequestStatus.nada);
    expectedPendingHints = makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, () =>
      PendingReadHint.nada(MAX_PUBLIC_DATA_READS_PER_TX),
    );
    expectedLeafDataHints = makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, () =>
      LeafDataReadHint.nada(MAX_PUBLIC_DATA_READS_PER_TX),
    );
  });

  it('returns empty hints', () => {
    buildAndCheckHints();
  });

  it('builds hints for reading pending values', () => {
    publicDataReads[0] = makePublicDataRead(77, 7777);
    publicDataReads[1] = makePublicDataRead(99, 9999);
    publicDataReads[2] = makePublicDataRead(55, 5555);
    expectedStatuses[0] = ReadRequestStatus.pending(0);
    expectedStatuses[1] = ReadRequestStatus.pending(1);
    expectedStatuses[2] = ReadRequestStatus.pending(2);
    expectedPendingHints[0] = makePendingHint(0, 1);
    expectedPendingHints[1] = makePendingHint(1, 2);
    expectedPendingHints[2] = makePendingHint(2, 0);

    buildAndCheckHints();
  });

  it('builds hints for reading settled or uninitialized values', () => {
    publicDataReads[0] = makePublicDataRead(33, 300);
    publicDataReads[1] = makePublicDataRead(77, 0);
    publicDataReads[2] = makePublicDataRead(55, 500);
    publicDataReads[3] = makePublicDataRead(11, 100);
    expectedStatuses[0] = ReadRequestStatus.settled(0);
    expectedStatuses[1] = ReadRequestStatus.settled(1);
    expectedStatuses[2] = ReadRequestStatus.settled(2);
    expectedStatuses[3] = ReadRequestStatus.settled(3);
    expectedLeafDataHints[0] = makeLeafDataHint(0, 2);
    expectedLeafDataHints[1] = makeLeafDataHint(1, 4);
    expectedLeafDataHints[2] = makeLeafDataHint(2, 3);
    expectedLeafDataHints[3] = makeLeafDataHint(3, 0);

    buildAndCheckHints();
  });

  it('builds hints for reading pending and settled values', () => {
    publicDataReads[0] = makePublicDataRead(55, 500);
    publicDataReads[1] = makePublicDataRead(55, 5555);
    publicDataReads[2] = makePublicDataRead(77, 0);
    publicDataReads[3] = makePublicDataRead(11, 100);
    publicDataReads[4] = makePublicDataRead(99, 9999);
    publicDataReads[5] = makePublicDataRead(77, 7777);
    publicDataReads[6] = makePublicDataRead(11, 100);
    expectedStatuses[0] = ReadRequestStatus.settled(0);
    expectedStatuses[1] = ReadRequestStatus.pending(0);
    expectedStatuses[2] = ReadRequestStatus.settled(1);
    expectedStatuses[3] = ReadRequestStatus.settled(2);
    expectedStatuses[4] = ReadRequestStatus.pending(1);
    expectedStatuses[5] = ReadRequestStatus.pending(2);
    expectedStatuses[6] = ReadRequestStatus.settled(3);
    expectedPendingHints[0] = makePendingHint(1, 0);
    expectedPendingHints[1] = makePendingHint(4, 2);
    expectedPendingHints[2] = makePendingHint(5, 1);
    expectedLeafDataHints[0] = makeLeafDataHint(0, 3);
    expectedLeafDataHints[1] = makeLeafDataHint(2, 4);
    expectedLeafDataHints[2] = makeLeafDataHint(3, 0);
    expectedLeafDataHints[3] = makeLeafDataHint(6, 0);

    buildAndCheckHints();
  });

  it('throws if reading unknown slot', () => {
    publicDataReads[0] = makePublicDataRead(123, 100);
    expect(() => buildHints()).toThrow('Cannot find a pending write or a data hint for the read request.');
  });

  it('throws if reading unknown value', () => {
    publicDataReads[0] = makePublicDataRead(11, 1111);
    expect(() => buildHints()).toThrow('Value being read does not match existing public data or pending writes.');
  });
});
