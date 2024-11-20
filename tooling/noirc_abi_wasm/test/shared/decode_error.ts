import { Abi } from '@noir-lang/noirc_abi';

export const FAKE_FIELD_SELECTOR = '1';
export const FAKE_TUPLE_SELECTOR = '2';
export const FAKE_FMT_STRING_SELECTOR = '3';
export const FAKE_STRUCT_SELECTOR = '4';
export const FAKE_STRING_SELECTOR = '5';

export const SAMPLE_FMT_STRING = 'hello {a}';
export const SAMPLE_STRING = 'hello world';

export const abi: Abi = {
  parameters: [
    {
      name: 'foo',
      type: { kind: 'array', length: 2, type: { kind: 'field' } },
      visibility: 'private',
    },
  ],
  return_type: null,
  error_types: {
    [FAKE_FIELD_SELECTOR]: {
      error_kind: 'custom',
      kind: 'field',
    },
    [FAKE_TUPLE_SELECTOR]: {
      error_kind: 'custom',
      kind: 'tuple',
      fields: [{ kind: 'field' }, { kind: 'field' }],
    },
    [FAKE_FMT_STRING_SELECTOR]: {
      error_kind: 'fmtstring',
      length: SAMPLE_FMT_STRING.length,
      item_types: [{ kind: 'field' }],
    },
    [FAKE_STRUCT_SELECTOR]: {
      error_kind: 'custom',
      kind: 'struct',
      path: 'foo',
      fields: [
        { name: 'a', type: { kind: 'field' } },
        { name: 'b', type: { kind: 'field' } },
      ],
    },
    [FAKE_STRING_SELECTOR]: {
      error_kind: 'string',
      string: SAMPLE_STRING,
    },
  },
};
