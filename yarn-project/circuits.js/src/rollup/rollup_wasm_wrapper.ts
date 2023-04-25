import {
  BaseRollupInputs,
  BaseOrMergeRollupPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  MergeRollupInputs,
  CircuitError,
} from '../index.js';
import { uint8ArrayToNum } from '../utils/serialize.js';
import { CircuitsWasm } from '../wasm/circuits_wasm.js';

export class RollupWasmWrapper {
  constructor(private wasm: CircuitsWasm) {}

  public simulateBaseRollup(baseRollupInputs: BaseRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    return this.callWasm('base_rollup__sim', baseRollupInputs, BaseOrMergeRollupPublicInputs);
  }

  public simulateMergeRollup(mergeRollupInputs: MergeRollupInputs): Promise<BaseOrMergeRollupPublicInputs> {
    return this.callWasm('merge_rollup__sim', mergeRollupInputs, BaseOrMergeRollupPublicInputs);
  }

  public simulateRootRollup(rootRollupInputs: RootRollupInputs): Promise<RootRollupPublicInputs> {
    return this.callWasm('root_rollup__sim', rootRollupInputs, RootRollupPublicInputs);
  }

  // Adapted from yarn-project/circuits.js/src/tests/expectSerialize.ts
  private async callWasm<T>(
    method: string,
    input: Buffer | { toBuffer: () => Buffer },
    outputType: { fromBuffer: (b: Buffer) => T },
  ): Promise<T> {
    const wasm = this.wasm;
    const inputBuf: Buffer = input instanceof Buffer ? input : input.toBuffer();

    // Allocate memory for the input buffer and the pointer to the pointer to the output buffer
    const inputBufPtr = wasm.call('bbmalloc', inputBuf.length);
    wasm.writeMemory(inputBufPtr, inputBuf);

    if (method === 'merge_rollup__sim') {
      // TODO: Remove this once base/root also return circuit failures
      const outputBufSizePtr = wasm.call('bbmalloc', 4);
      const outputBufPtrPtr = wasm.call('bbmalloc', 4);
      // Run and read outputs
      const circuitFailureBufPtr = await wasm.asyncCall(method, inputBufPtr, outputBufSizePtr, outputBufPtrPtr);
      if (circuitFailureBufPtr == 0) {
        // C++ returned a null pointer i.e. circuit didn't have an error
        const outputBufSize = uint8ArrayToNum(wasm.getMemorySlice(outputBufSizePtr, outputBufSizePtr + 4));
        const outputBufPtr = uint8ArrayToNum(wasm.getMemorySlice(outputBufPtrPtr, outputBufPtrPtr + 4));
        const outputBuf = Buffer.from(wasm.getMemorySlice(outputBufPtr, outputBufPtr + outputBufSize));
        const output = outputType.fromBuffer(outputBuf);

        // Free memory
        wasm.call('bbfree', circuitFailureBufPtr);
        wasm.call('bbfree', outputBufPtr);
        wasm.call('bbfree', outputBufPtrPtr);
        wasm.call('bbfree', inputBufPtr);

        return output;
      } else {
        // CircuitError struct is structured as:
        // 1st 16 bits after the `circuitFailureBufPtr` - error code (enum uint16)
        // Next 32 bits - error message size
        // Next `error message size` bytes - error message.
        // So need to first extract the error message size so we know how much memory to read for the entire error struct.
        const errorMessageSizeBuffer = Buffer.from(
          wasm.getMemorySlice(circuitFailureBufPtr + 2, circuitFailureBufPtr + 2 + 4),
        );
        const errorMessageSize = errorMessageSizeBuffer.readUint32BE();
        // Now extract the entire `CircuitError` struct:
        const errorBuf = Buffer.from(
          wasm.getMemorySlice(circuitFailureBufPtr, circuitFailureBufPtr + 2 + 4 + errorMessageSize),
        );
        const err = CircuitError.fromBuffer(errorBuf);

        // Free memory
        wasm.call('bbfree', circuitFailureBufPtr);

        return Promise.reject(`Circuit failed with code ${err.code} - ${err.message}`);
      }
    } else {
      const outputBufPtrPtr = wasm.call('bbmalloc', 4);
      // Run and read outputs
      const outputBufSize = await wasm.asyncCall(method, inputBufPtr, outputBufPtrPtr);
      const outputBufPtr = uint8ArrayToNum(wasm.getMemorySlice(outputBufPtrPtr, outputBufPtrPtr + 4));
      const outputBuf = Buffer.from(wasm.getMemorySlice(outputBufPtr, outputBufPtr + outputBufSize));
      const output = outputType.fromBuffer(outputBuf);

      // Free memory
      wasm.call('bbfree', outputBufPtr);
      wasm.call('bbfree', outputBufPtrPtr);
      wasm.call('bbfree', inputBufPtr);

      return output;
    }
  }
}
