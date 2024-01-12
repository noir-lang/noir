import { FunctionL2Logs } from './function_l2_logs.js';

describe('FunctionL2Logs', () => {
  it('can encode L2Logs to buffer and back', () => {
    const l2Logs = FunctionL2Logs.random(42);

    const buffer = l2Logs.toBuffer();
    const recovered = FunctionL2Logs.fromBuffer(buffer);

    expect(recovered).toEqual(l2Logs);
  });

  it('getSerializedLength returns the correct length', () => {
    const l2Logs = FunctionL2Logs.random(42);

    const buffer = l2Logs.toBuffer();
    const recovered = FunctionL2Logs.fromBuffer(buffer);

    expect(recovered.getSerializedLength()).toEqual(buffer.length);
  });
});
