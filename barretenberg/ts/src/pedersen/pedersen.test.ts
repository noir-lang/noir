import { Pedersen } from './pedersen.js';
import { Timer } from '../benchmark/timer.js';
import { Fr } from '../types/index.js';

describe('pedersen sync', () => {
  it('pedersenHash', async () => {
    const pedersen = await Pedersen.new();
    const result = pedersen.pedersenHash([new Fr(4n).toBuffer(), new Fr(8n).toBuffer()], 7);
    expect(result).toEqual(
      new Fr(2152386650411553803409271316104075950536496387580531018130718456431861859990n).toBuffer(),
    );
  });

  it('pedersenCommit', async () => {
    const pedersen = await Pedersen.new();
    const result = pedersen.pedersenCommit([new Fr(4n).toBuffer(), new Fr(8n).toBuffer(), new Fr(12n).toBuffer()]);
    expect(result).toEqual([
      new Fr(18374309251862457296563484909553154519357910650678202211610516068880120638872n).toBuffer(),
      new Fr(2572141322478528249692953821523229170092797347760799983831061874108357705739n).toBuffer(),
    ]);
  });

  it.skip('pedersenCommit perf test', async () => {
    const pedersen = await Pedersen.new();
    const loops = 1000;
    const fields = Array.from({ length: loops * 2 }).map(() => Fr.random());
    const t = new Timer();
    for (let i = 0; i < loops; ++i) {
      pedersen.pedersenCommit([fields[i * 2].toBuffer(), fields[i * 2 + 1].toBuffer()]);
    }
    console.log(t.us() / loops);
  });
});
