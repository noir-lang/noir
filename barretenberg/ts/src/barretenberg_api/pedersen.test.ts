import { Barretenberg } from '../barretenberg/index.js';
import { Fr, Point } from '../types/index.js';

describe('pedersen', () => {
  let api: Barretenberg;

  beforeAll(async () => {
    api = await Barretenberg.new(1);
    await api.pedersenHashInit();
  }, 30000);

  afterAll(async () => {
    await api.destroy();
  });

  it('pedersenHashWithHashIndex', async () => {
    const result = await api.pedersenHashWithHashIndex([new Fr(4n), new Fr(8n)], 7);
    expect(result).toEqual(new Fr(2152386650411553803409271316104075950536496387580531018130718456431861859990n));
  });

  it('pedersenCommit', async () => {
    const result = await api.pedersenCommit([new Fr(4n), new Fr(8n), new Fr(12n)]);
    expect(result).toEqual(
      new Point(
        new Fr(18374309251862457296563484909553154519357910650678202211610516068880120638872n),
        new Fr(2572141322478528249692953821523229170092797347760799983831061874108357705739n),
      ),
    );
  });
});
