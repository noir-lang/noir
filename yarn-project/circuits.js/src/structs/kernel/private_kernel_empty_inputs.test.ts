import { Fr } from '@aztec/foundation/fields';

import { makeHeader } from '../../tests/factories.js';
import { PrivateKernelEmptyInputData } from './private_kernel_empty_inputs.js';

describe('PrivateKernelEmptyInputData', () => {
  it('serializes and deserializes', () => {
    const obj = new PrivateKernelEmptyInputData(makeHeader(), Fr.random(), Fr.random(), Fr.random());
    expect(PrivateKernelEmptyInputData.fromString(obj.toString())).toEqual(obj);
  });
});
