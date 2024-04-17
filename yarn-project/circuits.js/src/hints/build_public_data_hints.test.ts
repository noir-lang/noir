import { makeTuple } from '@aztec/foundation/array';
import { Fr } from '@aztec/foundation/fields';
import { type Tuple } from '@aztec/foundation/serialize';

import {
  MAX_PUBLIC_DATA_HINTS,
  MAX_PUBLIC_DATA_READS_PER_TX,
  type MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
} from '../constants.gen.js';
import { PublicDataRead, PublicDataTreeLeafPreimage, PublicDataUpdateRequest } from '../structs/index.js';
import { buildPublicDataHints } from './build_public_data_hints.js';

class ExpectedHint {
  constructor(public leafSlot: number, public value: number, public matchOrLowLeafSlot: number) {}

  static empty() {
    return new ExpectedHint(0, 0, 0);
  }

  toExpectedObject() {
    return expect.objectContaining({
      leafSlot: new Fr(this.leafSlot),
      value: new Fr(this.value),
      leafPreimage: expect.objectContaining({ slot: new Fr(this.matchOrLowLeafSlot) }),
    });
  }
}

describe('buildPublicDataHints', () => {
  let publicDataReads: Tuple<PublicDataRead, typeof MAX_PUBLIC_DATA_READS_PER_TX>;
  let publicDataUpdateRequests: Tuple<PublicDataUpdateRequest, typeof MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>;
  let expectedHints: Tuple<ExpectedHint, typeof MAX_PUBLIC_DATA_HINTS>;

  const publicDataLeaves = [
    new PublicDataTreeLeafPreimage(new Fr(22), new Fr(200), new Fr(33), 0n),
    new PublicDataTreeLeafPreimage(new Fr(11), new Fr(100), new Fr(22), 0n),
    new PublicDataTreeLeafPreimage(new Fr(0), new Fr(0), new Fr(11), 0n),
  ];

  const makePublicDataRead = (leafSlot: number, value: number) => new PublicDataRead(new Fr(leafSlot), new Fr(value));
  const makePublicDataWrite = (leafSlot: number, value: number) =>
    new PublicDataUpdateRequest(new Fr(leafSlot), new Fr(value));

  const oracle = {
    getMatchOrLowPublicDataMembershipWitness: (leafSlot: bigint) => {
      const leafPreimage = publicDataLeaves.find(l => l.slot.toBigInt() <= leafSlot);
      return { membershipWitness: {}, leafPreimage } as any;
    },
  };

  const buildHints = () => buildPublicDataHints(oracle, publicDataReads, publicDataUpdateRequests);

  const buildAndCheckHints = async () => {
    const hints = await buildHints();
    const partialHints = expectedHints.map(h => h.toExpectedObject());
    expect(hints).toEqual(partialHints);
  };

  beforeEach(() => {
    publicDataReads = makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataRead.empty);
    publicDataUpdateRequests = makeTuple(MAX_PUBLIC_DATA_READS_PER_TX, PublicDataUpdateRequest.empty);
    expectedHints = makeTuple(MAX_PUBLIC_DATA_HINTS, ExpectedHint.empty);
  });

  it('returns empty hints', async () => {
    await buildAndCheckHints();
  });

  it('builds hints for reads for uninitialized slots', async () => {
    publicDataReads[0] = makePublicDataRead(12, 0);
    publicDataReads[1] = makePublicDataRead(39, 0);
    expectedHints[0] = new ExpectedHint(12, 0, 11);
    expectedHints[1] = new ExpectedHint(39, 0, 22);

    await buildAndCheckHints();
  });

  it('builds hints for reads for initialized slots', async () => {
    publicDataReads[0] = makePublicDataRead(22, 200);
    publicDataReads[1] = makePublicDataRead(11, 100);
    expectedHints[0] = new ExpectedHint(22, 200, 22);
    expectedHints[1] = new ExpectedHint(11, 100, 11);

    await buildAndCheckHints();
  });

  it('builds hints for writes to uninitialized slots', async () => {
    publicDataUpdateRequests[0] = makePublicDataWrite(5, 500);
    publicDataUpdateRequests[1] = makePublicDataWrite(17, 700);
    expectedHints[0] = new ExpectedHint(5, 0, 0);
    expectedHints[1] = new ExpectedHint(17, 0, 11);

    await buildAndCheckHints();
  });

  it('builds hints for writes to initialized slots', async () => {
    publicDataUpdateRequests[0] = makePublicDataWrite(11, 111);
    publicDataUpdateRequests[1] = makePublicDataWrite(22, 222);
    expectedHints[0] = new ExpectedHint(11, 100, 11);
    expectedHints[1] = new ExpectedHint(22, 200, 22);

    await buildAndCheckHints();
  });

  it('builds hints for mixed reads and writes', async () => {
    publicDataReads[0] = makePublicDataRead(22, 200);
    publicDataReads[1] = makePublicDataRead(12, 0);
    publicDataReads[2] = makePublicDataRead(39, 0);
    publicDataReads[3] = makePublicDataRead(11, 100);
    publicDataUpdateRequests[0] = makePublicDataWrite(11, 111);
    publicDataUpdateRequests[1] = makePublicDataWrite(5, 500);
    publicDataUpdateRequests[2] = makePublicDataWrite(17, 700);
    publicDataUpdateRequests[3] = makePublicDataWrite(22, 222);
    expectedHints[0] = new ExpectedHint(22, 200, 22);
    expectedHints[1] = new ExpectedHint(12, 0, 11);
    expectedHints[2] = new ExpectedHint(39, 0, 22);
    expectedHints[3] = new ExpectedHint(11, 100, 11);
    expectedHints[4] = new ExpectedHint(5, 0, 0);
    expectedHints[5] = new ExpectedHint(17, 0, 11);

    await buildAndCheckHints();
  });
});
