import { L2BlockL2Logs } from './l2_block_l2_logs.js';

describe('L2BlockL2Logs', () => {
  it('can encode L2Logs to buffer and back', () => {
    const l2Logs = L2BlockL2Logs.random(3, 6, 2);

    const buffer = l2Logs.toBuffer();
    const recovered = L2BlockL2Logs.fromBuffer(buffer);

    expect(recovered).toEqual(l2Logs);
  });

  it('getSerializedLength returns the correct length', () => {
    const l2Logs = L2BlockL2Logs.random(3, 6, 2);

    const buffer = l2Logs.toBuffer();
    const recovered = L2BlockL2Logs.fromBuffer(buffer);

    expect(recovered.getSerializedLength()).toEqual(buffer.length);
  });

  it('serializes to and from JSON', () => {
    const l2Logs = L2BlockL2Logs.random(3, 6, 2);
    const json = l2Logs.toJSON();
    const recovered = L2BlockL2Logs.fromJSON(json);
    expect(recovered).toEqual(l2Logs);
  });
});
