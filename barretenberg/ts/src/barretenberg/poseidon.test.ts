import { BarretenbergSync } from './index.js';
import { Timer } from '../benchmark/timer.js';
import { Fr } from '../types/index.js';

describe('poseidon sync', () => {
  let api: BarretenbergSync;

  beforeAll(async () => {
    api = await BarretenbergSync.new();
  });

  it('poseidonHash', () => {
    const result = api.poseidon2Hash([new Fr(4n), new Fr(8n)]);
    expect(result).toMatchSnapshot();
  });

  it('poseidonHash perf test', () => {
    const loops = 1000;
    const fields = Array.from({ length: loops * 2 }).map(() => Fr.random());
    const t = new Timer();
    for (let i = 0; i < loops; ++i) {
      api.poseidon2Hash([fields[i * 2], fields[i * 2 + 1]]);
    }
    const us = t.us() / loops;
    console.log(`Executed ${loops} hashes at an average ${us}us / hash`);
  });

  it('poseidonHashes perf test', () => {
    const loops = 10;
    const numHashesPerLoop = 1024;
    const fields = Array.from({ length: numHashesPerLoop * 2 }).map(() => Fr.random());
    const t = new Timer();
    for (let i = 0; i < loops; ++i) {
      api.poseidon2Hashes(fields);
    }
    const us = t.us() / (numHashesPerLoop * loops);
    console.log(`Executed ${numHashesPerLoop * loops} hashes at an average ${us}us / hash`);
  });
});
