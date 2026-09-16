import { Abi, InputMap } from '@noir-lang/noirc_abi';

export const abi: Abi = {
  abi_version: 1,
  parameters: [
    {
      name: 'foo',
      type: { kind: 'array', length: 2, type: { kind: 'field' } },
      visibility: 'private',
    },
  ],
  return_type: null,
  error_types: {},
};

export const inputs: InputMap = {
  foo: '1',
};
