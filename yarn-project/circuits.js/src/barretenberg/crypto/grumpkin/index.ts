import { BarretenbergSync } from '@aztec/bb.js';
import { Fr, type GrumpkinScalar, Point } from '@aztec/foundation/fields';

/**
 * Grumpkin elliptic curve operations.
 */
export class Grumpkin {
  private wasm = BarretenbergSync.getSingleton().getWasm();

  // prettier-ignore
  static generator = Point.fromBuffer(Buffer.from([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xcf, 0x13, 0x5e, 0x75, 0x06, 0xa4, 0x5d, 0x63,
    0x2d, 0x27, 0x0d, 0x45, 0xf1, 0x18, 0x12, 0x94, 0x83, 0x3f, 0xc4, 0x8d, 0x82, 0x3f, 0x27, 0x2c,
  ]));

  /**
   * Point generator
   * @returns The generator for the curve.
   */
  public generator(): Point {
    return Grumpkin.generator;
  }

  /**
   * Multiplies a point by a scalar (adds the point `scalar` amount of times).
   * @param point - Point to multiply.
   * @param scalar - Scalar to multiply by.
   * @returns Result of the multiplication.
   */
  public mul(point: Point, scalar: GrumpkinScalar): Point {
    this.wasm.writeMemory(0, point.toBuffer());
    this.wasm.writeMemory(64, scalar.toBuffer());
    this.wasm.call('ecc_grumpkin__mul', 0, 64, 96);
    return Point.fromBuffer(Buffer.from(this.wasm.getMemorySlice(96, 160)));
  }

  /**
   * Add two points.
   * @param a - Point a in the addition
   * @param b - Point b to add to a
   * @returns Result of the addition.
   */
  public add(a: Point, b: Point): Point {
    this.wasm.writeMemory(0, a.toBuffer());
    this.wasm.writeMemory(64, b.toBuffer());
    this.wasm.call('ecc_grumpkin__add', 0, 64, 128);
    return Point.fromBuffer(Buffer.from(this.wasm.getMemorySlice(128, 192)));
  }

  /**
   * Multiplies a set of points by a scalar.
   * @param points - Points to multiply.
   * @param scalar - Scalar to multiply by.
   * @returns Points multiplied by the scalar.
   */
  public batchMul(points: Point[], scalar: GrumpkinScalar) {
    const concatenatedPoints: Buffer = Buffer.concat(points.map(point => point.toBuffer()));
    const pointsByteLength = points.length * Point.SIZE_IN_BYTES;

    const mem = this.wasm.call('bbmalloc', pointsByteLength * 2);

    this.wasm.writeMemory(mem, concatenatedPoints);
    this.wasm.writeMemory(0, scalar.toBuffer());
    this.wasm.call('ecc_grumpkin__batch_mul', mem, 0, points.length, mem + pointsByteLength);

    const result: Buffer = Buffer.from(
      this.wasm.getMemorySlice(mem + pointsByteLength, mem + pointsByteLength + pointsByteLength),
    );
    this.wasm.call('bbfree', mem);

    const parsedResult: Point[] = [];
    for (let i = 0; i < pointsByteLength; i += 64) {
      parsedResult.push(Point.fromBuffer(result.subarray(i, i + 64)));
    }
    return parsedResult;
  }

  /**
   * Gets a random field element.
   * @returns Random field element.
   */
  public getRandomFr(): Fr {
    this.wasm.call('ecc_grumpkin__get_random_scalar_mod_circuit_modulus', 0);
    return Fr.fromBuffer(Buffer.from(this.wasm.getMemorySlice(0, 32)));
  }

  /**
   * Converts a 512 bits long buffer to a field.
   * @param uint512Buf - The buffer to convert.
   * @returns Buffer representation of the field element.
   */
  public reduce512BufferToFr(uint512Buf: Buffer): Fr {
    this.wasm.writeMemory(0, uint512Buf);
    this.wasm.call('ecc_grumpkin__reduce512_buffer_mod_circuit_modulus', 0, 64);
    return Fr.fromBuffer(Buffer.from(this.wasm.getMemorySlice(64, 96)));
  }
}
