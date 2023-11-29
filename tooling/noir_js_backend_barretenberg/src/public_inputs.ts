import { Abi, WitnessMap } from '@noir-lang/types';

export function flattenPublicInputs(publicInputs: WitnessMap): string[] {
  const publicInputIndices = [...publicInputs.keys()].sort((a, b) => a - b);
  const flattenedPublicInputs = publicInputIndices.map((index) => publicInputs.get(index) as string);
  return flattenedPublicInputs;
}

export function flattenPublicInputsAsArray(publicInputs: WitnessMap): Uint8Array {
  const flatPublicInputs = flattenPublicInputs(publicInputs);
  const flattenedPublicInputs = flatPublicInputs.map(hexToUint8Array);
  return flattenUint8Arrays(flattenedPublicInputs);
}

export function deflattenPublicInputs(flattenedPublicInputs: Uint8Array, abi: Abi): WitnessMap {
  const publicInputSize = 32;
  const chunkedFlattenedPublicInputs: Uint8Array[] = [];

  for (let i = 0; i < flattenedPublicInputs.length; i += publicInputSize) {
    const publicInput = flattenedPublicInputs.slice(i, i + publicInputSize);
    chunkedFlattenedPublicInputs.push(publicInput);
  }

  const return_value_witnesses = abi.return_witnesses;
  const public_parameters = abi.parameters.filter((param) => param.visibility === 'public');
  const public_parameter_witnesses: number[] = public_parameters.flatMap((param) =>
    abi.param_witnesses[param.name].flatMap((witness_range) =>
      Array.from({ length: witness_range.end - witness_range.start }, (_, i) => witness_range.start + i),
    ),
  );

  // We now have an array of witness indices which have been deduplicated and sorted in ascending order.
  // The elements of this array should correspond to the elements of `flattenedPublicInputs` so that we can build up a `WitnessMap`.
  const public_input_witnesses = [...new Set(public_parameter_witnesses.concat(return_value_witnesses))].sort(
    (a, b) => a - b,
  );

  const publicInputs: WitnessMap = new Map();
  public_input_witnesses.forEach((witness_index, index) => {
    const witness_value = uint8ArrayToHex(chunkedFlattenedPublicInputs[index]);
    publicInputs.set(witness_index, witness_value);
  });

  return publicInputs;
}

function flattenUint8Arrays(arrays: Uint8Array[]): Uint8Array {
  const totalLength = arrays.reduce((acc, val) => acc + val.length, 0);
  const result = new Uint8Array(totalLength);

  let offset = 0;
  for (const arr of arrays) {
    result.set(arr, offset);
    offset += arr.length;
  }

  return result;
}

function uint8ArrayToHex(buffer: Uint8Array): string {
  const hex: string[] = [];

  buffer.forEach(function (i) {
    let h = i.toString(16);
    if (h.length % 2) {
      h = '0' + h;
    }
    hex.push(h);
  });

  return '0x' + hex.join('');
}

function hexToUint8Array(hex: string): Uint8Array {
  const sanitised_hex = BigInt(hex).toString(16).padStart(64, '0');

  const len = sanitised_hex.length / 2;
  const u8 = new Uint8Array(len);

  let i = 0;
  let j = 0;
  while (i < len) {
    u8[i] = parseInt(sanitised_hex.slice(j, j + 2), 16);
    i += 1;
    j += 2;
  }

  return u8;
}
