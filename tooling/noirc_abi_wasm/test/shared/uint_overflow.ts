import { Abi, InputMap } from '@noir-lang/noirc_abi';

export const abi: Abi = {
  parameters: [
    {
      name: 'foo',
      type: { kind: 'integer', sign: 'unsigned', width: 32 },
      visibility: 'private',
    },
  ],
  param_witnesses: { foo: [{ start: 1, end: 2 }] },
  return_type: null,
  return_witnesses: [],
  error_types: {},
};

export const inputs: InputMap = {
  foo: `0x${(1n << 38n).toString(16)}`,
};
