import { createDebugLogger } from '@aztec/foundation/log';

import { GrumpkinScalar, type Point } from '../../../index.js';
import { Grumpkin } from './index.js';

const debug = createDebugLogger('bb:grumpkin_test');

describe('grumpkin', () => {
  let grumpkin!: Grumpkin;

  beforeAll(() => {
    grumpkin = new Grumpkin();
  });

  it('should correctly perform scalar muls', () => {
    const exponent = GrumpkinScalar.random();

    const numPoints = 2048;

    const inputPoints: Point[] = [];
    for (let i = 0; i < numPoints; ++i) {
      inputPoints.push(grumpkin.mul(Grumpkin.generator, GrumpkinScalar.random()));
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
