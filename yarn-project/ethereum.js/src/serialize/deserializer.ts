import {
  deserializeArrayFromVector,
  deserializeBigInt,
  deserializeBool,
  deserializeBufferFromVector,
  deserializeInt32,
  deserializeUInt32,
} from './free_funcs.js';

export type DeserializeFn<T> = (buf: Buffer, offset: number) => { elem: T; adv: number };

export class Deserializer {
  constructor(private buf: Buffer, private offset = 0) {}

  public bool() {
    return this.exec(deserializeBool) ? true : false;
  }

  public uInt32() {
    return this.exec(deserializeUInt32);
  }

  public int32() {
    return this.exec(deserializeInt32);
  }

  public bigInt(width = 32) {
    return this.exec((buf: Buffer, offset: number) => deserializeBigInt(buf, offset, width));
  }

  public vector() {
    return this.exec(deserializeBufferFromVector);
  }

  public buffer(width: number) {
    const buf = this.buf.slice(this.offset, this.offset + width);
    this.offset += width;
    return buf;
  }

  public string() {
    return this.vector().toString();
  }

  public date() {
    return new Date(Number(this.bigInt(8)));
  }

  public deserializeArray<T>(fn: DeserializeFn<T>) {
    return this.exec((buf: Buffer, offset: number) => deserializeArrayFromVector(fn, buf, offset));
  }

  public exec<T>(fn: DeserializeFn<T>): T {
    const { elem, adv } = fn(this.buf, this.offset);
    this.offset += adv;
    return elem;
  }

  public getOffset() {
    return this.offset;
  }
}
