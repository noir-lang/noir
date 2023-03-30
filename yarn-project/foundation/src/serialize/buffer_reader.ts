import { Fr, Fq } from '../fields/fields.js';

export class BufferReader {
  private index: number;
  constructor(private buffer: Buffer, offset = 0) {
    this.index = offset;
  }

  public static asReader(bufferOrReader: Buffer | BufferReader) {
    return Buffer.isBuffer(bufferOrReader) ? new BufferReader(bufferOrReader) : bufferOrReader;
  }

  public readNumber(): number {
    this.index += 4;
    return this.buffer.readUint32BE(this.index - 4);
  }

  public readBoolean(): boolean {
    this.index += 1;
    return Boolean(this.buffer.at(this.index - 1));
  }

  public readBytes(n: number): Buffer {
    this.index += n;
    return Buffer.from(this.buffer.subarray(this.index - n, this.index));
  }

  public readFr(): Fr {
    return Fr.fromBuffer(this);
  }

  public readFq(): Fq {
    return Fq.fromBuffer(this);
  }

  public readNumberVector(): number[] {
    return this.readVector({
      fromBuffer: (reader: BufferReader) => reader.readNumber(),
    });
  }

  public readVector<T>(itemDeserializer: { fromBuffer: (reader: BufferReader) => T }): T[] {
    const size = this.readNumber();
    const result = new Array<T>(size);
    for (let i = 0; i < size; i++) {
      result[i] = itemDeserializer.fromBuffer(this);
    }
    return result;
  }

  public readArray<T>(
    size: number,
    itemDeserializer: {
      fromBuffer: (reader: BufferReader) => T;
    },
  ): T[] {
    const result = new Array<T>(size);
    for (let i = 0; i < size; i++) {
      result[i] = itemDeserializer.fromBuffer(this);
    }
    return result;
  }

  public readObject<T>(deserializer: { fromBuffer: (reader: BufferReader) => T }): T {
    return deserializer.fromBuffer(this);
  }

  public peekBytes(n?: number): Buffer {
    return this.buffer.subarray(this.index, n ? this.index + n : undefined);
  }

  public readString(): string {
    return this.readBuffer().toString();
  }

  public readBuffer(): Buffer {
    const size = this.readNumber();
    return this.readBytes(size);
  }

  public readMap<T>(deserializer: { fromBuffer: (reader: BufferReader) => T }): { [key: string]: T } {
    const numEntries = this.readNumber();
    const map: { [key: string]: T } = {};
    for (let i = 0; i < numEntries; i++) {
      const key = this.readString();
      const value = this.readObject<T>(deserializer);
      map[key] = value;
    }
    return map;
  }
}
