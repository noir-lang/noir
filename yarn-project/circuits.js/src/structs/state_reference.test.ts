import { randomInt } from '@aztec/foundation/crypto';

import { STATE_REFERENCE_LENGTH } from '../constants.gen.js';
import { makeStateReference } from '../tests/factories.js';
import { StateReference } from './state_reference.js';

describe('StateReference', () => {
  let state: StateReference;

  beforeAll(() => {
    state = makeStateReference(randomInt(1000));
  });

  it('serializes to buffer and deserializes it back', () => {
    const buffer = state.toBuffer();
    const res = StateReference.fromBuffer(buffer);
    expect(res).toEqual(state);
  });

  it('serializes to field array and deserializes it back', () => {
    const fieldArray = state.toFields();
    const res = StateReference.fromFields(fieldArray);
    expect(res).toEqual(state);
  });

  it('number of fields matches constant', () => {
    const fields = state.toFields();
    expect(fields.length).toBe(STATE_REFERENCE_LENGTH);
  });
});
