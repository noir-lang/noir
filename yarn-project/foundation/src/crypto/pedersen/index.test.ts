import { toBufferBE } from '../../bigint-buffer/index.js';
import { pedersenCommit, pedersenHashWithHashIndex } from './index.js';

describe('pedersen', () => {
  it('pedersen commit', () => {
    const r = pedersenCommit([toBufferBE(1n, 32), toBufferBE(1n, 32)]);
    expect(r).toEqual([
      Buffer.from('2f7a8f9a6c96926682205fb73ee43215bf13523c19d7afe36f12760266cdfe15', 'hex'),
      Buffer.from('01916b316adbbf0e10e39b18c1d24b33ec84b46daddf72f43878bcc92b6057e6', 'hex'),
    ]);
  });

  it('pedersen hash', () => {
    const r = pedersenHashWithHashIndex([toBufferBE(1n, 32), toBufferBE(1n, 32)]);
    expect(r).toEqual(Buffer.from('07ebfbf4df29888c6cd6dca13d4bb9d1a923013ddbbcbdc3378ab8845463297b', 'hex'));
  });

  it('pedersen hash with index', () => {
    const r = pedersenHashWithHashIndex([toBufferBE(1n, 32), toBufferBE(1n, 32)], 5);
    expect(r).toEqual(Buffer.from('1c446df60816b897cda124524e6b03f36df0cec333fad87617aab70d7861daa6', 'hex'));
  });
});
