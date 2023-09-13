import { BufferReader } from './buffer_reader.js';

export interface OutputType<T = any> {
  SIZE_IN_BYTES?: number;
  fromBuffer: (b: Uint8Array | BufferReader) => T;
}

export function BoolDeserializer(): OutputType {
  return {
    SIZE_IN_BYTES: 1,
    fromBuffer: (buf: Uint8Array | BufferReader) => {
      const reader = BufferReader.asReader(buf);
      return reader.readBoolean();
    },
  };
}

export function NumberDeserializer(): OutputType {
  return {
    SIZE_IN_BYTES: 4,
    fromBuffer: (buf: Uint8Array | BufferReader) => {
      const reader = BufferReader.asReader(buf);
      return reader.readNumber();
    },
  };
}

export function VectorDeserializer<T>(t: OutputType<T>): OutputType {
  return {
    fromBuffer: (buf: Uint8Array | BufferReader) => {
      const reader = BufferReader.asReader(buf);
      return reader.readVector(t);
    },
  };
}

export function BufferDeserializer(): OutputType {
  return {
    fromBuffer: (buf: Uint8Array | BufferReader) => {
      const reader = BufferReader.asReader(buf);
      return reader.readBuffer();
    },
  };
}

export function StringDeserializer(): OutputType {
  return {
    fromBuffer: (buf: Uint8Array | BufferReader) => {
      const reader = BufferReader.asReader(buf);
      return reader.readString();
    },
  };
}
