import { Abi, InputMap } from '@noir-lang/noirc_abi';

export const abi: Abi = {
  parameters: [
    {
      name: 'foo',
      type: { kind: 'field' },
      visibility: 'private',
    },
  ],
  param_witnesses: { foo: [{ start: 1, end: 3 }] },
  return_type: null,
  return_witnesses: [],
  error_types: {},
};

export const inputs: InputMap = {
  foo: ['1', '2'],
};
