import {
  deserializeArrayFromVector,
  deserializeBigInt,
  deserializeBool,
  deserializeBufferFromVector,
  deserializeInt32,
  deserializeUInt32,
} from './free_funcs.js';

// eslint-disable-next-line jsdoc/require-jsdoc
export type DeserializeFn<T> = (
  buf: Buffer,
  offset: number,
) => {
  // eslint-disable-next-line jsdoc/require-jsdoc
  elem: T;
  // eslint-disable-next-line jsdoc/require-jsdoc
  adv: number;
};

// eslint-disable-next-line jsdoc/require-jsdoc
export class Deserializer {
  constructor(private buf: Buffer, private offset = 0) {}

  // eslint-disable-next-line jsdoc/require-jsdoc
  public bool() {
    return this.exec(deserializeBool) ? true : false;
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public uInt32() {
    return this.exec(deserializeUInt32);
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public int32() {
    return this.exec(deserializeInt32);
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public bigInt(width = 32) {
    return this.exec((buf: Buffer, offset: number) => deserializeBigInt(buf, offset, width));
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public vector() {
    return this.exec(deserializeBufferFromVector);
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public buffer(width: number) {
    const buf = this.buf.slice(this.offset, this.offset + width);
    this.offset += width;
    return buf;
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public string() {
    return this.vector().toString();
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public date() {
    return new Date(Number(this.bigInt(8)));
  }
  // eslint-disable-next-line jsdoc/require-jsdoc
  public deserializeArray<T>(fn: DeserializeFn<T>) {
    return this.exec((buf: Buffer, offset: number) => deserializeArrayFromVector(fn, buf, offset));
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public exec<T>(fn: DeserializeFn<T>): T {
    const { elem, adv } = fn(this.buf, this.offset);
    this.offset += adv;
    return elem;
  }

  // eslint-disable-next-line jsdoc/require-jsdoc
  public getOffset() {
    return this.offset;
  }
}
