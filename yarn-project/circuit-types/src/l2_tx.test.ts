import { L2Tx } from './l2_tx.js';

describe('L2Tx', () => {
  it('convert to and from buffer', () => {
    const tx = L2Tx.random();
    const buf = tx.toBuffer();
    expect(L2Tx.fromBuffer(buf)).toEqual(tx);
  });

  it('converts to and from string', () => {
    const tx = L2Tx.random();
    const str = tx.toString();
    expect(L2Tx.fromString(str)).toEqual(tx);
  });
});
