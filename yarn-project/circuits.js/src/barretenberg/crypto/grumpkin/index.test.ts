import { createDebugLogger } from '@aztec/foundation/log';

import { CircuitsWasm, Point, PrivateKey } from '../../../index.js';
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
    const exponent = PrivateKey.random();

    const numPoints = 2048;

    const inputPoints: Point[] = [];
    for (let i = 0; i < numPoints; ++i) {
      inputPoints.push(grumpkin.mul(Grumpkin.generator, PrivateKey.random()));
    }

    const start = new Date().getTime();
    const outputPoints = grumpkin.batchMul(inputPoints, exponent);
    debug(`batch mul in: ${new Date().getTime() - start}ms`);

    const start2 = new Date().getTime();
    for (let i = 0; i < numPoints; ++i) {
      grumpkin.mul(inputPoints[i], exponent);
    }
    debug(`regular mul in: ${new Date().getTime() - start2}ms`);

    for (let i = 0; i < numPoints; ++i) {
      const lhs = outputPoints[i];
      const rhs = grumpkin.mul(inputPoints[i], exponent);
      expect(lhs).toEqual(rhs);
    }
  });
});
