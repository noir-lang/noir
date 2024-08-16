import { Buffer32 } from '@aztec/foundation/buffer';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

/**Viem Signature
 *
 * A version of the Signature class that uses `0x${string}` values for r and s rather than
 * Buffer32s
 */
export type ViemSignature = {
  r: `0x${string}`;
  s: `0x${string}`;
  v: number;
  isEmpty: boolean;
};

/**
 * Signature
 *
 * Contains a signature split into it's primary components (r,s,v)
 */
export class Signature {
  constructor(
    /** The r value of the signature */
    public readonly r: Buffer32,
    /** The s value of the signature */
    public readonly s: Buffer32,
    /** The v value of the signature */
    public readonly v: number,
    /** Does this struct store an empty signature */
    public readonly isEmpty: boolean = false,
  ) {}

  static fromBuffer(buf: Buffer | BufferReader): Signature {
    const reader = BufferReader.asReader(buf);
    const r = reader.readObject(Buffer32);
    const s = reader.readObject(Buffer32);
    const v = reader.readNumber();

    const isEmpty = r.isZero() && s.isZero();

    return new Signature(r, s, v, isEmpty);
  }

  toBuffer(): Buffer {
    return serializeToBuffer([this.r, this.s, this.v]);
  }

  static empty(): Signature {
    return new Signature(Buffer32.ZERO, Buffer32.ZERO, 0, true);
  }

  to0xString(): `0x${string}` {
    return `0x${this.r.toString()}${this.s.toString()}${this.v.toString(16)}`;
  }

  /**
   * A seperate method exists for this as when signing locally with viem, as when
   * parsing from viem, we can expect the v value to be a u8, rather than our
   * default serialization of u32
   */
  static from0xString(sig: `0x${string}`): Signature {
    const buf = Buffer.from(sig.slice(2), 'hex');
    const reader = BufferReader.asReader(buf);

    const r = reader.readObject(Buffer32);
    const s = reader.readObject(Buffer32);
    const v = reader.readUInt8();

    const isEmpty = r.isZero() && s.isZero();

    return new Signature(r, s, v, isEmpty);
  }

  /**
   * Return the signature with `0x${string}` encodings for r and s
   */
  toViemSignature(): ViemSignature {
    return {
      r: this.r.to0xString(),
      s: this.s.to0xString(),
      v: this.v,
      isEmpty: this.isEmpty,
    };
  }
}
