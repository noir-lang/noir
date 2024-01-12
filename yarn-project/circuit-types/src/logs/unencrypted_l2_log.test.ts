import { UnencryptedL2Log } from './unencrypted_l2_log.js';

describe('UnencryptedL2Log', () => {
  it('can encode L2Logs to buffer and back', () => {
    const l2Logs = UnencryptedL2Log.random();

    const buffer = l2Logs.toBuffer();
    const recovered = UnencryptedL2Log.fromBuffer(buffer);

    expect(recovered).toEqual(l2Logs);
  });
});
