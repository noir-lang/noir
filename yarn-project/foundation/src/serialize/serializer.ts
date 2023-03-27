import { serializeBufferArrayToVector } from './index.js';
import {
  boolToByte,
  numToInt32BE,
  numToUInt32BE,
  serializeBigInt,
  serializeBufferToVector,
  serializeDate,
} from './free_funcs.js';

// export type DeserializeFn<T> = (buf: Buffer, offset: number) => { elem: T; adv: number };

export class Serializer {
  private buf: Buffer[] = [];

  constructor() {}

  public bool(bool: boolean) {
    this.buf.push(boolToByte(bool));
  }

  public uInt32(num: number) {
    this.buf.push(numToUInt32BE(num));
  }

  public int32(num: number) {
    this.buf.push(numToInt32BE(num));
  }

  public bigInt(num: bigint) {
    this.buf.push(serializeBigInt(num));
  }

  /**
   * The given buffer is of variable length. Prefixes the buffer with its length.
   */
  public vector(buf: Buffer) {
    this.buf.push(serializeBufferToVector(buf));
  }

  /**
   * Directly serializes a buffer that maybe of fixed, or variable length.
   * It is assumed the corresponding deserialize function will handle variable length data, thus the length
   * does not need to be prefixed here.
   * If serializing a raw, variable length buffer, use vector().
   */
  public buffer(buf: Buffer) {
    this.buf.push(buf);
  }

  public string(str: string) {
    this.vector(Buffer.from(str));
  }

  public date(date: Date) {
    this.buf.push(serializeDate(date));
  }

  public getBuffer() {
    return Buffer.concat(this.buf);
  }

  public serializeArray<T>(arr: T[]) {
    this.buf.push(serializeBufferArrayToVector(arr.map((e: any) => e.toBuffer())));
  }
}
