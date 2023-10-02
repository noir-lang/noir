export const abi = {
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

export const inputs = {
  foo: '1',
};
