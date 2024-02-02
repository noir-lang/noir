import { STATE_REFERENCE_LENGTH } from '../constants.gen.js';
import { makeStateReference } from '../tests/factories.js';
import { StateReference } from './state_reference.js';

describe('StateReference', () => {
  let state: StateReference;

  beforeAll(() => {
    const randomInt = Math.floor(Math.random() * 1000);
    state = makeStateReference(randomInt);
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
