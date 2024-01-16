/* eslint-disable camelcase */
const typeMap: { [key: string]: string } = {
  in_ptr: 'Ptr',
  out_ptr: 'Ptr',
  'bb::fr::in_buf': 'Fr',
  'bb::fr::vec_in_buf': 'Fr[]',
  'fr::in_buf': 'Fr',
  'fr::out_buf': 'Fr',
  'fr::vec_in_buf': 'Fr[]',
  'fr::vec_out_buf': 'Fr[]',
  'fq::in_buf': 'Fq',
  'fq::out_buf': 'Fq',
  'fq::vec_in_buf': 'Fq[]',
  'fq::vec_out_buf': 'Fq[]',
  'const uint8_t *': 'Uint8Array',
  'uint8_t **': 'Uint8Array',
  in_str_buf: 'string',
  out_str_buf: 'string',
  in_buf32: 'Buffer32',
  out_buf32: 'Buffer32',
  'uint32_t *': 'number',
  'const uint32_t *': 'number',
  'affine_element::in_buf': 'Point',
  'affine_element::out_buf': 'Point',
  'const bool *': 'boolean',
  'bool *': 'boolean',
  'multisig::MultiSigPublicKey::vec_in_buf': 'Buffer128[]',
  'multisig::MultiSigPublicKey::out_buf': 'Buffer128',
  'multisig::RoundOnePublicOutput::vec_in_buf': 'Buffer128[]',
  'multisig::RoundOnePublicOutput::out_buf': 'Buffer128',
  'multisig::RoundOnePrivateOutput::in_buf': 'Buffer128',
  'multisig::RoundOnePrivateOutput::out_buf': 'Buffer128',
};

const deserializerMap: { [key: string]: string } = {
  out_ptr: 'Ptr',
  'fr::out_buf': 'Fr',
  'fr::vec_out_buf': 'VectorDeserializer(Fr)',
  'fq::out_buf': 'Fq',
  'fq::vec_out_buf': 'VectorDeserializer(Fq)',
  'uint8_t **': 'BufferDeserializer()',
  out_str_buf: 'StringDeserializer()',
  out_buf32: 'Buffer32',
  'uint32_t *': 'NumberDeserializer()',
  'affine_element::out_buf': 'Point',
  'bool *': 'BoolDeserializer()',
  'multisig::MultiSigPublicKey::out_buf': 'Buffer128',
  'multisig::RoundOnePublicOutput::out_buf': 'Buffer128',
  'multisig::RoundOnePrivateOutput::out_buf': 'Buffer128',
};

export function mapType(type: string) {
  if (typeMap[type]) {
    return typeMap[type];
  }
  throw new Error(`Unknown type: ${type}`);
}

export const mapRustType = mapType;

export function mapDeserializer(type: string) {
  if (deserializerMap[type]) {
    return deserializerMap[type];
  }
  throw new Error(`Unknown deserializer for type: ${type}`);
}
