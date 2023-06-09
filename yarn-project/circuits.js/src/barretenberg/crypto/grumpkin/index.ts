import { IWasmModule } from '@aztec/foundation/wasm';
import { CircuitsWasm } from '../../../index.js';

/**
 * Grumpkin elliptic curve operations.
 */
export class Grumpkin {
  /**
   * Creates a new Grumpkin instance.
   * @returns New Grumpkin instance.
   */
  public static async new() {
    return new this(await CircuitsWasm.get());
  }

  constructor(private wasm: IWasmModule) {}

  // prettier-ignore
  static generator = Buffer.from([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xcf, 0x13, 0x5e, 0x75, 0x06, 0xa4, 0x5d, 0x63,
    0x2d, 0x27, 0x0d, 0x45, 0xf1, 0x18, 0x12, 0x94, 0x83, 0x3f, 0xc4, 0x8d, 0x82, 0x3f, 0x27, 0x2c,
  ]);

  /**
   * Multiplies a point by a scalar (adds the point `scalar` amount of time).
   * @param point - Point to multiply.
   * @param scalar - Scalar to multiply by.
   * @returns Result of the multiplication.
   */
  public mul(point: Uint8Array, scalar: Uint8Array) {
    this.wasm.writeMemory(0, point);
    this.wasm.writeMemory(64, scalar);
    this.wasm.call('ecc_grumpkin__mul', 0, 64, 96);
    return Buffer.from(this.wasm.getMemorySlice(96, 160));
  }

  /**
   * Multiplies a set of points by a scalar.
   * @param points - Points to multiply.
   * @param scalar - Scalar to multiply by.
   * @param numPoints - Number of points in the points array.
   * @returns Points multiplied by the scalar.
   */
  public batchMul(points: Uint8Array, scalar: Uint8Array, numPoints: number) {
    const mem = this.wasm.call('bbmalloc', points.length * 2);

    this.wasm.writeMemory(mem, points);
    this.wasm.writeMemory(0, scalar);
    this.wasm.call('ecc_grumpkin__batch_mul', mem, 0, numPoints, mem + points.length);

    const result: Buffer = Buffer.from(
      this.wasm.getMemorySlice(mem + points.length, mem + points.length + points.length),
    );
    this.wasm.call('bbfree', mem);
    return result;
  }

  /**
   * Gets a random field element.
   * @returns Random field element.
   */
  public getRandomFr() {
    this.wasm.call('ecc_grumpkin__get_random_scalar_mod_circuit_modulus', 0);
    return Buffer.from(this.wasm.getMemorySlice(0, 32));
  }

  /**
   * Converts a 512 bits long buffer to a field.
   * @param uint512Buf - The buffer to convert.
   * @returns Buffer representation of the field element.
   */
  public reduce512BufferToFr(uint512Buf: Buffer) {
    this.wasm.writeMemory(0, uint512Buf);
    this.wasm.call('ecc_grumpkin__reduce512_buffer_mod_circuit_modulus', 0, 64);
    return Buffer.from(this.wasm.getMemorySlice(64, 96));
  }
}
