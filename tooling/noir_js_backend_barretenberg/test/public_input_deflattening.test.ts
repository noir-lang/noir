import { Abi } from '@noir-lang/types';
import { expect } from 'chai';
import { witnessMapToPublicInputs, publicInputsToWitnessMap } from '../src/public_inputs.js';

const abi: Abi = {
  parameters: [
    {
      name: 'array_with_returned_element',
      type: {
        kind: 'array',
        type: {
          kind: 'field',
        },
        length: 10,
      },
      visibility: 'private',
    },
    {
      name: 'pub_field',
      type: {
        kind: 'field',
      },
      visibility: 'public',
    },
  ],
  param_witnesses: {
    array_with_returned_element: [
      {
        start: 1,
        end: 11,
      },
    ],
    pub_field: [
      {
        start: 11,
        end: 12,
      },
    ],
  },
  return_type: {
    abi_type: {
      kind: 'tuple',
      fields: [
        {
          kind: 'field',
        },
        {
          kind: 'field',
        },
        {
          kind: 'field',
        },
      ],
    },
    visibility: 'public',
  },
  return_witnesses: [2, 13, 13],
  error_types: {},
};

it('flattens a witness map in order of its witness indices', async () => {
  // Note that these are not in ascending order. This means that if we read from `witness_map` in insertion order
  // then the witness values will be sorted incorrectly.
  const public_input_indices = [2, 13, 11];

  const witness_map = new Map(
    public_input_indices.map((witness_index) => [
      witness_index,
      '0x' + BigInt(witness_index).toString(16).padStart(64, '0'),
    ]),
  );

  const flattened_public_inputs = witnessMapToPublicInputs(witness_map);
  expect(flattened_public_inputs).to.be.deep.eq([
    '0x0000000000000000000000000000000000000000000000000000000000000002',
    '0x000000000000000000000000000000000000000000000000000000000000000b',
    '0x000000000000000000000000000000000000000000000000000000000000000d',
  ]);
});

it('recovers the original witness map when deflattening a public input array', async () => {
  // Note that these are not in ascending order. This means that if we read from `witness_map` in insertion order
  // then the witness values will be sorted incorrectly.
  const public_input_indices = [2, 13, 11];

  const witness_map = new Map(
    public_input_indices.map((witness_index) => [
      witness_index,
      '0x' + BigInt(witness_index).toString(16).padStart(64, '0'),
    ]),
  );

  const flattened_public_inputs = witnessMapToPublicInputs(witness_map);
  const deflattened_public_inputs = publicInputsToWitnessMap(flattened_public_inputs, abi);

  expect(deflattened_public_inputs).to.be.deep.eq(witness_map);
});
