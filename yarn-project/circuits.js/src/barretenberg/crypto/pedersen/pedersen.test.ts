import { CircuitsWasm } from '@aztec/circuits.js';

import { Buffer } from 'buffer';

import { pedersenGetHashTree } from './pedersen.js';

describe('pedersen', () => {
  let barretenbergWasm!: CircuitsWasm;
  const values: Buffer[] = [];

  beforeAll(async () => {
    barretenbergWasm = await CircuitsWasm.get();

    // TODO was originally 2 ** 12
    for (let i = 0; i < 2 ** 2; ++i) {
      const v = Buffer.alloc(32, 0);
      v.writeUInt32LE(i, 0);
      values[i] = v;
    }
  });

  it('hasher_consistency_and_benchmark', () => {
    // const start1 = new Date().getTime();
    const result = pedersenGetHashTree(barretenbergWasm, values);
    // const end1 = new Date().getTime() - start1;

    // console.log(`Single hasher: ~${end1 / values.length}ms / value`);
    // TODO more than smoke-test this
    expect(result.length).toBe(values.length * 2 - 1);
    // TODO
    // console.log(`Pooled hasher: ~${end2 / values.length}ms / value`);
    // console.log(`Pooled improvement: ${(end1 / end2).toFixed(2)}x`);
    // expect(poolResults).toEqual(singleResults);
    // await pool.destroy();
  });
});
