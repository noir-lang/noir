import { FunctionSelector } from '@aztec/foundation/abi';
import { setupCustomSnapshotSerializers, updateInlineTestData } from '@aztec/foundation/testing';

import { FUNCTION_DATA_LENGTH } from '../constants.gen.js';
import { FunctionData } from './function_data.js';

describe('FunctionData', () => {
  let functionData: FunctionData;

  beforeAll(() => {
    setupCustomSnapshotSerializers(expect);
    functionData = new FunctionData(new FunctionSelector(123), true);
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

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/noir-protocol-circuits/crates/types/src/abis/function_data.nr',
      'test_data_empty_hash',
      hash.toString(),
    );
  });
});
