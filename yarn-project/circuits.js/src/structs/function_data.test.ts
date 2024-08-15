import { FunctionSelector } from '@aztec/foundation/abi';
import { setupCustomSnapshotSerializers } from '@aztec/foundation/testing';

import { FUNCTION_DATA_LENGTH } from '../constants.gen.js';
import { FunctionData } from './function_data.js';

describe('FunctionData', () => {
  let functionData: FunctionData;

  beforeAll(() => {
    setupCustomSnapshotSerializers(expect);
    functionData = new FunctionData(new FunctionSelector(123), /*isPrivate=*/ true);
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
});
