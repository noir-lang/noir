import { Abi, InputMap } from '@noir-lang/noirc_abi';

export const abi: Abi = {
  parameters: [
    {
      name: 'foo',
      type: { kind: 'array', length: 2, type: { kind: 'field' } },
      visibility: 'private',
    },
  ],
  param_witnesses: { foo: [1, 2] },
  return_type: null,
  return_witnesses: [],
};

export const inputs: InputMap = {
  foo: '1',
};
