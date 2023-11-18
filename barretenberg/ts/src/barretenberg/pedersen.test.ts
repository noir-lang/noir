import { BarretenbergSync } from './index.js';
import { Timer } from '../benchmark/timer.js';
import { Fr } from '../types/index.js';

describe('pedersen sync', () => {
  let api: BarretenbergSync;

  beforeAll(async () => {
    api = await BarretenbergSync.new();
  });

  it('pedersenHash', () => {
    const result = api.pedersenHash([new Fr(4n), new Fr(8n)], 7);
    expect(result).toMatchSnapshot();
  });

  it('pedersenHashBuffer', () => {
    const input = Buffer.alloc(123);
    input.writeUint32BE(321, 0);
    input.writeUint32BE(456, 119);
    const r = api.pedersenHashBuffer(input, 0);
    expect(r).toMatchSnapshot();
  });

  it('pedersenCommit', () => {
    const result = api.pedersenCommit([new Fr(4n), new Fr(8n), new Fr(12n)]);
    expect(result).toMatchSnapshot();
  });

  it.skip('pedersenCommit perf test', () => {
    const loops = 1000;
    const fields = Array.from({ length: loops * 2 }).map(() => Fr.random());
    const t = new Timer();
    for (let i = 0; i < loops; ++i) {
      api.pedersenCommit([fields[i * 2], fields[i * 2 + 1]]);
    }
    console.log(t.us() / loops);
  });
});
