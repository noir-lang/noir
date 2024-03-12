import { randomInt } from '@aztec/foundation/crypto';

import { PARTIAL_STATE_REFERENCE_LENGTH } from '../constants.gen.js';
import { makePartialStateReference } from '../tests/factories.js';
import { PartialStateReference } from './partial_state_reference.js';

describe('PartialStateReference', () => {
  let partial: PartialStateReference;

  beforeAll(() => {
    partial = makePartialStateReference(randomInt(1000));
  });

  it('serializes to buffer and deserializes it back', () => {
    const buffer = partial.toBuffer();
    const res = PartialStateReference.fromBuffer(buffer);
    expect(res).toEqual(partial);
  });

  it('serializes to field array and deserializes it back', () => {
    const fieldArray = partial.toFields();
    const res = PartialStateReference.fromFields(fieldArray);
    expect(res).toEqual(partial);
  });

  it('number of fields matches constant', () => {
    const fields = partial.toFields();
    expect(fields.length).toBe(PARTIAL_STATE_REFERENCE_LENGTH);
  });
});
