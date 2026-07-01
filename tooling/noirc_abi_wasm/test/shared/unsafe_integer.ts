import { Abi } from '@noir-lang/noirc_abi';

export const abi: Abi = {
  parameters: [
    {
      name: 'foo',
      type: { kind: 'field' },
      visibility: 'private',
    },
  ],
  return_type: null,
  error_types: {},
};

// `Number.MAX_SAFE_INTEGER + 2` is not representable as a JavaScript `number`: it rounds to
// `2^53` before reaching wasm. Encoding it would silently produce a witness for a different field
// element, so the encoder must reject it and require a string instead.
export const unsafeInteger = Number.MAX_SAFE_INTEGER + 2;

// A boxed `Number` and an object with a `toJSON` method both hide the unsafe value from a walk of
// the raw input object, but `JSON.stringify` (used to bridge the value into wasm) still surfaces
// the rounded number, so the encoder must reject these too.
export const boxedUnsafeInteger = new Number(unsafeInteger);
export const toJsonUnsafeInteger = {
  toJSON() {
    return unsafeInteger;
  },
};
