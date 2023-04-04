import { AztecAddress, EthAddress, Fr } from '@aztec/foundation';

export type ACVMField = `0x${string}`;
export type ACVMWitness = Map<number, ACVMField>;

export const ZERO_ACVM_FIELD: ACVMField = `0x${Buffer.alloc(32).toString('hex')}`;
export const ONE_ACVM_FIELD: ACVMField = `0x${'00'.repeat(31)}01`;

function adaptBufferSize(originalBuf: Buffer) {
  const buffer = Buffer.alloc(32);
  if (originalBuf.length > buffer.length) {
    throw new Error('Buffer does not fit in 32 bytes');
  }
  originalBuf.copy(buffer, buffer.length - originalBuf.length);
  return buffer;
}

export function toACVMField(value: AztecAddress | EthAddress | Fr | Buffer | boolean | number): `0x${string}` {
  if (typeof value === 'boolean') {
    return value ? ONE_ACVM_FIELD : ZERO_ACVM_FIELD;
  }

  let buffer;

  if (Buffer.isBuffer(value)) {
    buffer = value;
  } else if (typeof value === 'number') {
    buffer = Buffer.alloc(32);
    buffer.writeUInt32BE(value, 28);
  } else {
    buffer = value.toBuffer();
  }

  return `0x${adaptBufferSize(buffer).toString('hex')}`;
}

export function fromACVMField(field: `0x${string}`): Fr {
  const buffer = Buffer.from(field.slice(2), 'hex');
  return Fr.fromBuffer(buffer);
}
