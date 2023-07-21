import { randomBytes } from '@aztec/foundation/crypto';
import { createDebugLogger } from '@aztec/foundation/log';

import { CircuitsWasm } from '../../../index.js';
import { Grumpkin } from './index.js';

const debug = createDebugLogger('bb:grumpkin_test');

describe('grumpkin', () => {
  let barretenberg!: CircuitsWasm;
  let grumpkin!: Grumpkin;

  beforeAll(async () => {
    barretenberg = await CircuitsWasm.get();
    grumpkin = new Grumpkin(barretenberg);
  });

  it('should correctly perform scalar muls', () => {
    const exponent = randomBytes(32);

    const numPoints = 2048;

    const points: Buffer[] = [];
    for (let i = 0; i < numPoints; ++i) {
      points.push(grumpkin.mul(Grumpkin.generator, randomBytes(32)));
    }
    let pointBuf: Buffer = points[0];

    for (let i = 1; i < numPoints; ++i) {
      pointBuf = Buffer.concat([pointBuf, points[i]]);
    }

    const start = new Date().getTime();
    const result = grumpkin.batchMul(pointBuf, exponent, numPoints);
    debug(`batch mul in: ${new Date().getTime() - start}ms`);

    const start2 = new Date().getTime();
    for (let i = 0; i < numPoints; ++i) {
      grumpkin.mul(points[i], exponent);
    }
    debug(`regular mul in: ${new Date().getTime() - start2}ms`);

    for (let i = 0; i < numPoints; ++i) {
      const lhs: Buffer = Buffer.from(result.slice(i * 64, i * 64 + 64));
      const rhs = grumpkin.mul(points[i], exponent);
      expect(lhs).toEqual(rhs);
    }
  });
});
