import { BarretenbergWasmMain } from '../barretenberg_wasm/barretenberg_wasm_main/index.js';
import { numToUInt32BE, serializeBufferArrayToVector } from '../serialize/serialize.js';

export class Pedersen {
  constructor(public wasm: BarretenbergWasmMain) {}

  static async new() {
    const wasm = new BarretenbergWasmMain();
    await wasm.init(1);
    return new Pedersen(wasm);
  }

  pedersenHash(inputs: Uint8Array[], hashIndex = 0) {
    const SCRATCH_SPACE_SIZE = 1024;

    const data = serializeBufferArrayToVector(inputs);

    let inputPtr = 0;
    if (data.length > SCRATCH_SPACE_SIZE - 4) {
      inputPtr = this.wasm.call('bbmalloc', data.length);
    }
    this.wasm.writeMemory(inputPtr, data);
    this.wasm.writeMemory(SCRATCH_SPACE_SIZE - 4, numToUInt32BE(hashIndex));

    const outputPtr = 0;
    this.wasm.call('pedersen_hash', inputPtr, SCRATCH_SPACE_SIZE - 4, outputPtr);
    const hashOutput = this.wasm.getMemorySlice(0, 32);

    if (inputPtr !== 0) {
      this.wasm.call('bbfree', inputPtr);
    }

    return hashOutput;
  }

  pedersenCommit(inputs: Uint8Array[]) {
    const SCRATCH_SPACE_SIZE = 1024;

    const data = serializeBufferArrayToVector(inputs);

    let inputPtr = 0;
    if (data.length > SCRATCH_SPACE_SIZE) {
      inputPtr = this.wasm.call('bbmalloc', data.length);
    }
    this.wasm.writeMemory(inputPtr, data);

    const outputPtr = 0;
    this.wasm.call('pedersen_commit', inputPtr, outputPtr);
    const hashOutput = this.wasm.getMemorySlice(0, 64);

    if (inputPtr !== 0) {
      this.wasm.call('bbfree', inputPtr);
    }

    return [hashOutput.slice(0, 32), hashOutput.slice(32, 64)];
  }
}
