export class BufferReader {
  private index: number;
  constructor(private buffer: Uint8Array, offset = 0) {
    this.index = offset;
  }

  public static asReader(bufferOrReader: Uint8Array | BufferReader) {
    return bufferOrReader instanceof BufferReader ? bufferOrReader : new BufferReader(bufferOrReader);
  }

  public readNumber(): number {
    const dataView = new DataView(this.buffer.buffer, this.buffer.byteOffset + this.index, 4);
    this.index += 4;
    return dataView.getUint32(0, false);
  }

  public readBoolean(): boolean {
    this.index += 1;
    return Boolean(this.buffer.at(this.index - 1));
  }

  public readBytes(n: number): Uint8Array {
    this.index += n;
    return this.buffer.slice(this.index - n, this.index);
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

  public peekBytes(n?: number) {
    return this.buffer.subarray(this.index, n ? this.index + n : undefined);
  }

  public readString(): string {
    return new TextDecoder().decode(this.readBuffer());
  }

  public readBuffer(): Uint8Array {
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
