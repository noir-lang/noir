import { serializeToBuffer } from "../wasm/serialize.js";

/**
 * Function description for circuit.
 * @see abis/function_data.hpp
 */
export class FunctionData {
  constructor(
    public functionSelector: number,
    public isPrivate: true,
    public isConstructor: true
  ) {}
  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(
      this.functionSelector,
      this.isPrivate,
      this.isConstructor
    );
  }
}
