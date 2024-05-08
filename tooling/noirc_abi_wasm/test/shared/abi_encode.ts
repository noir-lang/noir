import { Abi, InputMap } from '@noir-lang/noirc_abi';

export const abi: Abi = {
  parameters: [
    { name: 'foo', type: { kind: 'field' }, visibility: 'private' },
    {
      name: 'bar',
      type: { kind: 'array', length: 2, type: { kind: 'field' } },
      visibility: 'private',
    },
  ],
  param_witnesses: { foo: [{ start: 1, end: 2 }], bar: [{ start: 2, end: 4 }] },
  return_type: null,
  return_witnesses: [],
  error_types: {},
};

export const inputs: InputMap = {
  foo: '1',
  bar: ['1', '2'],
};
