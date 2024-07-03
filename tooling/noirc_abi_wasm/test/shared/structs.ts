import { Abi, Field, InputMap } from '@noir-lang/noirc_abi';

export type MyStruct = {
  foo: Field;
};

export type MyNestedStruct = {
  foo: MyStruct;
};

export const abi: Abi = {
  parameters: [
    {
      name: 'struct_arg',
      type: { kind: 'struct', path: 'MyStruct', fields: [{ name: 'foo', type: { kind: 'field' } }] },
      visibility: 'private',
    },
    {
      name: 'struct_array_arg',
      type: {
        kind: 'array',
        type: {
          kind: 'struct',
          path: 'MyStruct',
          fields: [{ name: 'foo', type: { kind: 'field' } }],
        },
        length: 3,
      },
      visibility: 'private',
    },
    {
      name: 'nested_struct_arg',
      type: {
        kind: 'struct',
        path: 'MyNestedStruct',
        fields: [
          {
            name: 'foo',
            type: {
              kind: 'struct',
              path: 'MyStruct',
              fields: [{ name: 'foo', type: { kind: 'field' } }],
            },
          },
        ],
      },
      visibility: 'private',
    },
  ],
  return_type: null,
  error_types: {},
};

export const inputs: InputMap = {
  struct_arg: {
    foo: '1',
  },
  struct_array_arg: [
    {
      foo: '2',
    },
    {
      foo: '3',
    },
    {
      foo: '4',
    },
  ],
  nested_struct_arg: {
    foo: {
      foo: '5',
    },
  },
};
