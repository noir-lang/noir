import { randomInt } from '@aztec/foundation/crypto';

import { GLOBAL_VARIABLES_LENGTH } from '../constants.gen.js';
import { makeGlobalVariables } from '../tests/factories.js';
import { GlobalVariables } from './global_variables.js';

describe('GlobalVariables', () => {
  let state: GlobalVariables;

  beforeAll(() => {
    state = makeGlobalVariables(randomInt(1000));
  });

  it('serializes to buffer and deserializes it back', () => {
    const buffer = state.toBuffer();
    const res = GlobalVariables.fromBuffer(buffer);
    expect(res).toEqual(state);
  });

  it('serializes to field array and deserializes it back', () => {
    const fieldArray = state.toFields();
    const res = GlobalVariables.fromFields(fieldArray);
    expect(res).toEqual(state);
  });

  it('number of fields matches constant', () => {
    const fields = state.toFields();
    expect(fields.length).toBe(GLOBAL_VARIABLES_LENGTH);
  });
});
