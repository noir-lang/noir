import { FunctionSelector } from '@aztec/foundation/abi';

import { FUNCTION_DATA_LENGTH } from '../constants.gen.js';
import { FunctionData } from './function_data.js';

describe('FunctionData', () => {
  let functionData: FunctionData;

  beforeAll(() => {
    functionData = new FunctionData(new FunctionSelector(123), false, true, true);
  });

  it(`serializes to buffer and deserializes it back`, () => {
    const buffer = functionData.toBuffer();
    const res = FunctionData.fromBuffer(buffer);
    expect(res).toEqual(functionData);
    expect(res.isEmpty()).toBe(false);
  });

  it('number of fields matches constant', () => {
    const fields = functionData.toFields();
    expect(fields.length).toBe(FUNCTION_DATA_LENGTH);
  });

  it('computes empty inputs hash', () => {
    const data = FunctionData.empty();
    const hash = data.hash();
    expect(hash).toMatchSnapshot();

    // Value used in empty_hash test in private_circuit_public_inputs.nr
    // console.log("hash", hash.toString());
  });
});
