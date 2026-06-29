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
