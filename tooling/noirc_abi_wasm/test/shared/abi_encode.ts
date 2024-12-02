import { Abi, InputMap } from '@noir-lang/noirc_abi';

export const abi: Abi = {
  parameters: [
    { name: 'foo', type: { kind: 'field' }, visibility: 'private' },
    {
      name: 'bar',
      type: { kind: 'array', length: 3, type: { kind: 'integer', sign: 'signed', width: 32 } },
      visibility: 'private',
    },
  ],
  return_type: null,
  error_types: {},
};

export const inputs: InputMap = {
  foo: '1',
  bar: ['1', '2', '-1'],
};
