// TODO: Add type definitions for these

export const abi = {
  parameters: [
    { name: "foo", type: { kind: "field" }, visibility: "private" },
    {
      name: "bar",
      type: { kind: "array", length: 2, type: { kind: "field" } },
      visibility: "private",
    },
  ],
  param_witnesses: { foo: [1], bar: [2, 3] },
  return_type: null,
  return_witnesses: [],
};

export const inputs = {
  foo: "1",
  bar: ["1", "2"],
};
