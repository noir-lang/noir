import { strict as assert } from 'assert';

/*
 * A Buffer-like class that automatically advances the position.
 */
export class BufferCursor {
  constructor(private _buffer: Buffer, private _position: number = 0) {}

  public position(): number {
    return this._position;
  }

  public eof(): boolean {
    return this._position === this._buffer.length;
  }

  public bufferAtPosition(): Buffer {
    return this._buffer.subarray(this._position);
  }

  public advance(n: number): void {
    this._position += n;
    assert(n < this._buffer.length);
  }

  public readUint8(): number {
    const ret = this._buffer.readUint8(this._position);
    this._position += 1;
    return ret;
  }

  public readUint16LE(): number {
    const ret = this._buffer.readUint16LE(this._position);
    this._position += 2;
    return ret;
  }

  public readUint16BE(): number {
    const ret = this._buffer.readUint16BE(this._position);
    this._position += 2;
    return ret;
  }

  public readUint32LE(): number {
    const ret = this._buffer.readUint32LE(this._position);
    this._position += 4;
    return ret;
  }

  public readUint32BE(): number {
    const ret = this._buffer.readUint32BE(this._position);
    this._position += 4;
    return ret;
  }

  public readBigInt64LE(): bigint {
    const ret = this._buffer.readBigInt64LE(this._position);
    this._position += 8;
    return ret;
  }

  public readBigInt64BE(): bigint {
    const ret = this._buffer.readBigInt64BE(this._position);
    this._position += 8;
    return ret;
  }

  public writeUint8(v: number) {
    const ret = this._buffer.writeUint8(v, this._position);
    this._position += 1;
    return ret;
  }

  public writeUint16LE(v: number) {
    const ret = this._buffer.writeUint16LE(v, this._position);
    this._position += 2;
    return ret;
  }

  public writeUint16BE(v: number) {
    const ret = this._buffer.writeUint16BE(v, this._position);
    this._position += 2;
    return ret;
  }

  public writeUint32LE(v: number) {
    const ret = this._buffer.writeUint32LE(v, this._position);
    this._position += 4;
    return ret;
  }

  public writeUint32BE(v: number) {
    const ret = this._buffer.writeUint32BE(v, this._position);
    this._position += 4;
    return ret;
  }

  public writeBigInt64LE(v: bigint) {
    const ret = this._buffer.writeBigInt64LE(v, this._position);
    this._position += 8;
    return ret;
  }

  public writeBigInt64BE(v: bigint) {
    const ret = this._buffer.writeBigInt64BE(v, this._position);
    this._position += 8;
    return ret;
  }
}
