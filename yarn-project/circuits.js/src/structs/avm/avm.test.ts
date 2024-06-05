import { randomInt } from '@aztec/foundation/crypto';

import { makeAvmCircuitInputs } from '../../tests/factories.js';
import { AvmCircuitInputs } from './avm.js';

describe('Avm circuit inputs', () => {
  it(`serializes to buffer and deserializes it back`, () => {
    const avmCircuitInputs = makeAvmCircuitInputs(randomInt(2000));
    const buffer = avmCircuitInputs.toBuffer();
    const res = AvmCircuitInputs.fromBuffer(buffer);
    expect(res).toEqual(avmCircuitInputs);
    expect(res.isEmpty()).toBe(false);
  });
});
