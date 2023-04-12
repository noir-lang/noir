import {
  BaseRollupInputs,
  BaseOrMergeRollupPublicInputs,
  RootRollupInputs,
  RootRollupPublicInputs,
  MergeRollupInputs,
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
    input: { toBuffer: () => Buffer },
    outputType: { fromBuffer: (b: Buffer) => T },
  ): Promise<T> {
    const wasm = this.wasm;
    const inputBuf = input.toBuffer();

    // Allocate memory for the input buffer and the pointer to the pointer to the output buffer
    const inputBufPtr = wasm.call('bbmalloc', inputBuf.length);
    wasm.writeMemory(inputBufPtr, inputBuf);
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
