import { randomInt } from '@aztec/foundation/crypto';

import { makeAvmCircuitInputs, makeAvmExecutionHints, makeAvmHint } from '../../tests/factories.js';
import { AvmCircuitInputs, AvmExecutionHints, AvmHint } from './avm.js';

describe('Avm circuit inputs', () => {
  describe('AvmHint', () => {
    let avmHint: AvmHint;

    beforeAll(() => {
      avmHint = makeAvmHint(randomInt(1000));
    });

    it(`serializes to buffer and deserializes it back`, () => {
      const buffer = avmHint.toBuffer();
      const res = AvmHint.fromBuffer(buffer);
      expect(res).toEqual(avmHint);
      expect(res.isEmpty()).toBe(false);
    });
  });
  describe('AvmExecutionHints', () => {
    let avmExecutionHints: AvmExecutionHints;

    beforeAll(() => {
      avmExecutionHints = makeAvmExecutionHints(randomInt(1000));
    });

    it(`serializes to buffer and deserializes it back`, () => {
      const buffer = avmExecutionHints.toBuffer();
      const res = AvmExecutionHints.fromBuffer(buffer);
      expect(res).toEqual(avmExecutionHints);
      expect(res.isEmpty()).toBe(false);
    });
  });
  describe('AvmCircuitInputs', () => {
    let avmCircuitInputs: AvmCircuitInputs;

    beforeAll(() => {
      avmCircuitInputs = makeAvmCircuitInputs(randomInt(2000));
    });

    it(`serializes to buffer and deserializes it back`, () => {
      const buffer = avmCircuitInputs.toBuffer();
      const res = AvmCircuitInputs.fromBuffer(buffer);
      expect(res).toEqual(avmCircuitInputs);
      expect(res.isEmpty()).toBe(false);
    });
  });
});
