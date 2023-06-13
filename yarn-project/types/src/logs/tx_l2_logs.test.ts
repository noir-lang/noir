import { TxL2Logs } from './tx_l2_logs.js';

describe('TxL2Logs', () => {
  it('can encode L2Logs to buffer and back', () => {
    const l2Logs = TxL2Logs.random(6, 2);

    const buffer = l2Logs.toBuffer();
    const recovered = TxL2Logs.fromBuffer(buffer);

    expect(recovered).toEqual(l2Logs);
  });

  it('getSerializedLength returns the correct length', () => {
    const l2Logs = TxL2Logs.random(6, 2);

    const buffer = l2Logs.toBuffer();
    const recovered = TxL2Logs.fromBuffer(buffer);

    expect(recovered.getSerializedLength()).toEqual(buffer.length);
  });
});
